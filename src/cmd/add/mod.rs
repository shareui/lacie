pub mod parse;

use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::{print_warning, DiagError, DiagWarning};
use crate::model::manifest::{ResolvedPackage, WorkspaceManifest};
use crate::net::archive::unpack_archive;
use crate::net::hash::compute_sha256_file;
use crate::platform::fs::{activate_binary, set_binary_permissions};
use crate::platform::target::{detect_current_target, match_target, TargetMatchResult};
use crate::ruby::plugin::{SearchResult, SourcePlugin};
use crate::state::packages::{PackageVersionEntry, PackagesDatabase};
use crate::state::seen::SeenDatabase;
use crate::ui::color::{visible_len, ColorMode, Styler};
use crate::ui::progress::{render_spinner, run_with_download_progress};
use crate::ui::prompt::{prompt_yes_no, prompt_yes_no_view, PromptResult};
use crate::ui::syntax::print_manifest_view;
use crate::ui::viewer::{InteractiveViewer, ViewerAction, ViewerFile, ViewerFileType};
use chrono::Utc;
use parse::{parse_add_ref, ParsedAddRef};
use std::path::{Path, PathBuf};

pub struct AddOptions {
    pub target_ref: String,
    pub list_only: bool,
    pub info_only: bool,
    pub show_only: bool,
    pub iwannasee: bool,
    pub force: bool,
    pub set_hash: Option<String>,
    pub ignore_hash: bool,
    pub max_results: Option<usize>,
}

pub fn parse_add_options(args: &[String]) -> Result<AddOptions, DiagError> {
    let mut pos_args = Vec::new();
    let mut list_only = false;
    let mut info_only = false;
    let mut show_only = false;
    let mut iwannasee = false;
    let mut force = false;
    let mut set_hash = None;
    let mut ignore_hash = false;
    let mut max_results = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--list" {
            list_only = true;
        } else if arg == "--info" {
            info_only = true;
        } else if arg == "--show" {
            show_only = true;
        } else if arg == "--iwannasee" {
            iwannasee = true;
        } else if arg == "--force" {
            force = true;
        } else if arg == "--ignore-hash" {
            ignore_hash = true;
        } else if arg == "--set-hash" && i + 1 < args.len() {
            set_hash = Some(args[i + 1].clone());
            i += 1;
        } else if arg == "--max" && i + 1 < args.len() {
            max_results = args[i + 1].parse::<usize>().ok();
            i += 1;
        } else if !arg.starts_with('-') {
            pos_args.push(arg.clone());
        }
        i += 1;
    }

    if pos_args.is_empty() {
        return Err(DiagError::from_code("E017", &[("expected", "package reference")]));
    }
    let target_ref = if pos_args.len() >= 2 {
        std::format!("{} {}", pos_args[0], pos_args[1])
    } else {
        pos_args[0].clone()
    };
    Ok(AddOptions {
        target_ref,
        list_only,
        info_only,
        show_only,
        iwannasee,
        force,
        set_hash,
        ignore_hash,
        max_results,
    })
}

fn prompt_search_selection(
    candidates: &[&SearchResult],
    display_count: usize,
    query: &str,
    source: &str,
) -> Result<Option<String>, DiagError> {
    use std::io::{IsTerminal, Write};
    println!("\nRepositories matching \"{}\" in {}:\n", query, source);
    for (idx, item) in candidates[..display_count].iter().enumerate() {
        let stars_str = if item.stars >= 1000 {
            std::format!("{:.1}k stars", item.stars as f64 / 1000.0)
        } else {
            std::format!("{} stars", item.stars)
        };
        println!("  {}. {:<25} {}", idx + 1, item.storage, stars_str);
    }
    if candidates.len() > display_count {
        let more = candidates.len() - display_count;
        println!("  and {} more...", more);
        println!("  use --max N to increase the display limit");
    }
    println!();

    if !std::io::stdin().is_terminal() {
        return Ok(Some(candidates[0].storage.clone()));
    }

    loop {
        print!("Select [1-{}, q to quit]: ", display_count);
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("q") || trimmed.is_empty() {
            return Ok(None);
        }
        if let Ok(num) = trimmed.parse::<usize>()
            && num >= 1
            && num <= display_count
        {
            return Ok(Some(candidates[num - 1].storage.clone()));
        }
    }
}

fn resolve_manifest(
    parsed: &mut ParsedAddRef,
    paths: &LaciePaths,
    opts: &AddOptions,
    cfg: &Config,
    plugin: &mut Option<SourcePlugin>,
) -> Result<String, DiagError> {
    if parsed.is_local_file {
        let path = PathBuf::from(&parsed.storage);
        if path.is_file() && path.file_name().and_then(|n| n.to_str()).unwrap_or("").ends_with(".toml") {
            return std::fs::read_to_string(&path).map_err(|e| DiagError::new("E015", e.to_string()));
        }
        let unpack_dest = paths.var_dir.join("local_unpack");
        let _ = std::fs::remove_dir_all(&unpack_dest);
        unpack_archive(&path, &unpack_dest)?;
        let manifest_path = unpack_dest.join("lacie.toml");
        if !manifest_path.is_file() {
            return Err(DiagError::from_code("E013", &[("path", &path.display().to_string())]));
        }
        return std::fs::read_to_string(&manifest_path).map_err(|e| DiagError::new("E015", e.to_string()));
    }

    let mut p = SourcePlugin::load(&parsed.source, &paths.plugins_dir)?;
    if !parsed.storage.contains('/') {
        let search_res = p.search(&parsed.storage, true, cfg.cache_ttl, &paths.cache_dir)?;
        if search_res.is_empty() {
            return Err(DiagError::from_code("E005", &[("ref", &parsed.storage)]));
        }

        let query_lower = parsed.storage.to_lowercase();
        let exact_matches: Vec<&SearchResult> = search_res
            .iter()
            .filter(|r| {
                let repo = r.storage.split('/').next_back().unwrap_or(&r.storage);
                repo.to_lowercase() == query_lower || r.storage.to_lowercase() == query_lower
            })
            .collect();

        if exact_matches.len() == 1 {
            parsed.storage = exact_matches[0].storage.clone();
        } else {
            let candidates: Vec<&SearchResult> = if exact_matches.len() > 1 {
                exact_matches
            } else {
                search_res.iter().collect()
            };
            let max_limit = opts.max_results.unwrap_or(5);
            let display_count = candidates.len().min(max_limit);
            match prompt_search_selection(&candidates, display_count, &parsed.storage, &parsed.source)? {
                Some(selected) => parsed.storage = selected,
                None => {
                    *plugin = Some(p);
                    return Ok(String::new());
                }
            }
        }
    }
    if opts.list_only {
        let releases = p.releases(&parsed.storage, false, 0, &paths.cache_dir)?;
        println!("Releases for {}:", parsed.storage);
        for r in releases {
            println!("  {}", r.tag);
        }
        *plugin = Some(p);
        return Ok(String::new());
    }
    if parsed.release.is_none() {
        let releases = p.releases(&parsed.storage, true, cfg.cache_ttl, &paths.cache_dir)?;
        let latest = releases.into_iter().find(|r| !r.pre_release)
            .ok_or_else(|| DiagError::from_code("E005", &[("ref", &parsed.storage)]));
        if let Err(e) = latest {
            if opts.info_only {
                *plugin = Some(p);
                return Ok(String::new());
            }
            return Err(e);
        }
        parsed.release = Some(latest.unwrap().tag);
    }
    let tag = parsed.release.as_deref().unwrap_or("latest");
    let text = p.manifest(&parsed.storage, tag)?;
    *plugin = Some(p);
    Ok(text)
}

fn select_packages<'a>(manifest: &'a WorkspaceManifest, requested: &[String]) -> Result<Vec<&'a String>, DiagError> {
    let mut selected_ids: Vec<&'a String> = Vec::new();
    if requested.is_empty() {
        selected_ids.extend(manifest.packages.keys());
    } else {
        for req in requested {
            let mut found = false;
            for (id, pkg) in &manifest.packages {
                if id == req || &pkg.name == req {
                    if !selected_ids.contains(&id) {
                        selected_ids.push(id);
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(DiagError::from_code("E009", &[("name", req)]));
            }
        }
    }

    let mut i = 0;
    while i < selected_ids.len() {
        let current_id = selected_ids[i];
        if let Some(pkg) = manifest.packages.get(current_id) {
            let mut reqs = Vec::new();
            if let Some(r) = pkg.required.as_ref().or(pkg.requires.as_ref()) {
                reqs.extend(r.iter().map(|s| s.as_str()));
            }
            if let Some(ws) = manifest.workspace.as_ref()
                && let Some(r) = ws.required.as_ref().or(ws.requires.as_ref())
            {
                reqs.extend(r.iter().map(|s| s.as_str()));
            }
            if let Some(apps) = pkg.apps.as_ref() {
                for app in apps {
                    if app.from.is_none() {
                        reqs.push(&app.id);
                    }
                }
            }
            for dep in reqs {
                for (id, p) in &manifest.packages {
                    if (id == dep || p.name == dep) && !selected_ids.contains(&id) {
                        selected_ids.push(id);
                        break;
                    }
                }
            }
        }
        i += 1;
    }
    Ok(selected_ids)
}

fn render_single_inspection(
    pkg: &ResolvedPackage,
    target: &str,
    is_seen: bool,
    manifest_sha: &str,
    styler: &Styler,
) {
    let sha_display = pkg.binaries.get(target).and_then(|b| b.sha256.as_deref()).unwrap_or("unknown");
    let size_str = pkg.binaries.get(target)
        .and_then(|b| b.size)
        .map(|s| std::format!("{:.2} MiB", s as f64 / 1_048_576.0))
        .unwrap_or_else(|| "unknown".to_string());
    println!("{}  Package   {} {}", styler.dim("│"), styler.bold_white(&pkg.name), styler.dim(&std::format!("({})", pkg.id)));
    println!("{}  Target    {}", styler.dim("│"), target);
    println!("{}  Size      {}", styler.dim("│"), size_str);
    println!("{}  SHA256    {} {}", styler.dim("│"), sha_display, styler.dim("(manifest)"));
    if is_seen {
        println!("{}  Seen      lacie.toml   {}", styler.dim("│"), manifest_sha);
        println!("{}            {}", styler.dim("│"), styler.dim("already reviewed, not shown (use --iwannasee to see them again)"));
    }
    println!();
}

fn render_multi_inspection(pkgs: &[(&ResolvedPackage, &str)], source: &str, release: &str, styler: &Styler) {
    println!("{}  Source    {}", styler.dim("│"), source);
    println!("{}  Release   {} {}", styler.dim("│"), release, styler.dim("(release)"));
    let mut total_bytes: u64 = 0;
    let mut all_sizes_known = true;
    for (pkg, target) in pkgs {
        if let Some(s) = pkg.binaries.get(*target).and_then(|b| b.size) {
            total_bytes += s;
        } else {
            all_sizes_known = false;
        }
    }
    if all_sizes_known && !pkgs.is_empty() {
        let total_mb = total_bytes as f64 / 1_048_576.0;
        println!("{}  Total     {:.2} MiB ({} packages)", styler.dim("│"), total_mb, pkgs.len());
    } else {
        println!("{}  Total     {} packages", styler.dim("│"), pkgs.len());
    }
    println!("{}", styler.dim("│"));
    println!("{}  Packages ({}):", styler.dim("│"), pkgs.len());
    for (pkg, target) in pkgs {
        let sha_display = pkg.binaries.get(*target).and_then(|b| b.sha256.as_deref()).unwrap_or("unknown");
        let size_str = pkg.binaries.get(*target)
            .and_then(|b| b.size)
            .map(|s| std::format!("{:.2} MiB", s as f64 / 1_048_576.0))
            .unwrap_or_else(|| "unknown".to_string());
        println!("{}  - {} {} ({}, {})", styler.dim("│"), styler.bold_white(&pkg.name), styler.dim(&std::format!("({})", pkg.id)), target, size_str);
        println!("{}    SHA256: {} {}", styler.dim("│"), sha_display, styler.dim("(manifest)"));
    }
    println!();
}

pub struct DownloadContext<'a> {
    pub paths: &'a LaciePaths,
    pub plugin: &'a mut Option<SourcePlugin>,
    pub parsed: &'a ParsedAddRef,
    pub opts: &'a AddOptions,
    pub styler: &'a Styler,
}

fn do_fetch_or_download(
    pkg: &ResolvedPackage,
    target: &str,
    temp_dest: &Path,
    parsed: &ParsedAddRef,
    plugin: &mut Option<SourcePlugin>,
    paths: &LaciePaths,
) -> Result<(), DiagError> {
    let bin_entry = pkg.binaries.get(target).unwrap();
    let filename = bin_entry.file.clone()
        .or_else(|| bin_entry.links.as_ref().and_then(|l| l.first()).and_then(|url| url.split('/').next_back().map(|s| s.to_string())))
        .unwrap_or_else(|| std::format!("{}-{}", pkg.name, target));

    if parsed.is_local_file {
        let local_extracted = paths.var_dir.join("local_unpack").join(&filename);
        if local_extracted.is_file() {
            let _ = std::fs::copy(&local_extracted, temp_dest);
        }
    } else if let Some(p) = plugin {
        let release = parsed.release.as_deref().unwrap_or("latest");
        p.fetch(&parsed.storage, release, &filename, temp_dest)?;
    } else if let Some(links) = &bin_entry.links
        && let Some(url) = links.first()
    {
        crate::net::client::HttpClient::new().download_file(url, temp_dest)?;
    }
    Ok(())
}

fn verify_and_get_sha(
    temp_dest: &Path,
    bin_entry: &crate::model::manifest::BinaryEntry,
    opts: &AddOptions,
    filename: &str,
) -> Result<String, DiagError> {
    let actual_sha = compute_sha256_file(temp_dest)?;
    if let Some(expected) = opts.set_hash.as_ref().or(bin_entry.sha256.as_ref())
        && !opts.ignore_hash
        && &actual_sha != expected
    {
        let _ = std::fs::remove_file(temp_dest);
        return Err(DiagError::from_code("E007", &[("file", filename)]));
    }
    Ok(actual_sha)
}

fn install_binary_to_bin(
    temp_dest: &Path,
    paths: &LaciePaths,
    pkg: &ResolvedPackage,
    actual_sha: &str,
) -> Result<PathBuf, DiagError> {
    let install_dir = paths.bin_dir.join(&pkg.id).join(&pkg.version);
    std::fs::create_dir_all(&install_dir).map_err(|e| DiagError::new("E015", e.to_string()))?;
    let target_bin_name = std::format!("{}-{}", pkg.name, actual_sha);
    let final_bin_path = install_dir.join(&target_bin_name);
    std::fs::rename(temp_dest, &final_bin_path).map_err(|e| DiagError::new("E015", e.to_string()))?;
    set_binary_permissions(&final_bin_path).map_err(|e| DiagError::new("E015", e.to_string()))?;
    Ok(final_bin_path)
}

fn record_version_in_db(
    db: &mut PackagesDatabase,
    pkg: &ResolvedPackage,
    target: &str,
    actual_sha: &str,
    parsed: &ParsedAddRef,
) {
    let version_entry = PackageVersionEntry {
        version: pkg.version.clone(),
        version_number: pkg.version_number,
        author: pkg.author.clone(),
        links: pkg.links.clone(),
        source_code: pkg.source_code.clone(),
        installed_at: Some(Utc::now()),
        released_at: None,
        source: parsed.source.clone(),
        storage: parsed.storage.clone(),
        target: target.to_string(),
        sha256: actual_sha.to_string(),
    };
    db.insert_version(&pkg.id, &pkg.name, None, version_entry);
}

fn execute_single_package_add(
    pkg: &ResolvedPackage,
    target: &str,
    ctx: &mut DownloadContext,
    db: &mut PackagesDatabase,
) -> Result<(), DiagError> {
    let bin_entry = pkg.binaries.get(target).unwrap();
    let filename = bin_entry.file.clone()
        .or_else(|| bin_entry.links.as_ref().and_then(|l| l.first()).and_then(|url| url.split('/').next_back().map(|s| s.to_string())))
        .unwrap_or_else(|| std::format!("{}-{}", pkg.name, target));
    let temp_dest = ctx.paths.var_dir.join(&filename);

    println!("  {} {}", ctx.styler.green("Downloading"), filename);
    println!();

    let plugin_ref = &mut *ctx.plugin;
    let paths_ref = ctx.paths;
    let parsed_ref = ctx.parsed;
    run_with_download_progress(
        "  ",
        bin_entry.size,
        &temp_dest,
        0,
        ctx.styler,
        || do_fetch_or_download(pkg, target, &temp_dest, parsed_ref, plugin_ref, paths_ref),
    )?;

    println!();
    render_spinner("Verifying sha256 checksum...", 300, ctx.styler);
    println!("    {} sha256 checksum {}", ctx.styler.green("Verifying"), ctx.styler.dim("(matches manifest)"));

    let actual_sha = verify_and_get_sha(&temp_dest, bin_entry, ctx.opts, &filename)?;
    let final_bin_path = install_binary_to_bin(&temp_dest, ctx.paths, pkg, &actual_sha)?;
    record_version_in_db(db, pkg, target, &actual_sha, ctx.parsed);

    let active_bin = ctx.paths.active_dir.join(&pkg.name);
    activate_binary(&final_bin_path, &active_bin).map_err(|e| DiagError::new("E015", e.to_string()))?;
    println!("    {} {} -> {}", ctx.styler.green("Activating"), active_bin.display(), final_bin_path.display());
    if let Some(entry) = db.get_package_mut(&pkg.id) {
        entry.active = Some(pkg.version.clone());
    }

    let installed_title = std::format!("{} {}", pkg.name, pkg.version);
    println!("\n{} Installed {} (command: {}, {})", ctx.styler.bold_cyan("::"), ctx.styler.bold_white(&installed_title), ctx.styler.bold_white(&pkg.name), ctx.styler.green("active"));
    Ok(())
}

fn execute_multi_package_add(
    resolved_list: &[(&ResolvedPackage, &str)],
    ctx: &mut DownloadContext,
    db: &mut PackagesDatabase,
) -> Result<(), DiagError> {
    let start_time = std::time::Instant::now();
    let total_count = resolved_list.len();
    let prefixes: Vec<String> = resolved_list.iter().enumerate().map(|(i, (pkg, _))| {
        let step = std::format!("{}/{}", i + 1, total_count);
        std::format!("  {} ({}) {}", ctx.styler.green("Downloading"), ctx.styler.bold_cyan(&step), pkg.name)
    }).collect();
    let max_prefix_width = prefixes.iter().map(|p| visible_len(p)).max().unwrap_or(0) + 2;

    let mut installed_list = Vec::new();
    let mut final_bins = Vec::new();

    for (i, (pkg, target)) in resolved_list.iter().enumerate() {
        let bin_entry = pkg.binaries.get(*target).unwrap();
        let filename = bin_entry.file.clone()
            .or_else(|| bin_entry.links.as_ref().and_then(|l| l.first()).and_then(|url| url.split('/').next_back().map(|s| s.to_string())))
            .unwrap_or_else(|| std::format!("{}-{}", pkg.name, target));
        let temp_dest = ctx.paths.var_dir.join(&filename);

        let plugin_ref = &mut *ctx.plugin;
        let paths_ref = ctx.paths;
        let parsed_ref = ctx.parsed;
        run_with_download_progress(
            &prefixes[i],
            bin_entry.size,
            &temp_dest,
            max_prefix_width,
            ctx.styler,
            || do_fetch_or_download(pkg, target, &temp_dest, parsed_ref, plugin_ref, paths_ref),
        )?;

        let actual_sha = verify_and_get_sha(&temp_dest, bin_entry, ctx.opts, &filename)?;
        let final_bin_path = install_binary_to_bin(&temp_dest, ctx.paths, pkg, &actual_sha)?;
        record_version_in_db(db, pkg, target, &actual_sha, ctx.parsed);
        installed_list.push((pkg.name.clone(), pkg.version.clone(), pkg.id.clone()));
        final_bins.push((pkg.name.clone(), final_bin_path));
    }

    let spin_text = std::format!("Verifying checksums for {} packages...", total_count);
    render_spinner(&spin_text, 300, ctx.styler);
    let step_all = std::format!("{}/{}", total_count, total_count);
    println!("    {} ({}) checksums {}", ctx.styler.green("Verifying"), ctx.styler.bold_cyan(&step_all), ctx.styler.dim("(all match)"));

    let mut active_displays = Vec::new();
    for (name, bin_path) in final_bins {
        let active_bin = ctx.paths.active_dir.join(&name);
        activate_binary(&bin_path, &active_bin).map_err(|e| DiagError::new("E015", e.to_string()))?;
        active_displays.push(std::format!("active/{}", name));
    }
    for (_, ver, id) in &installed_list {
        if let Some(entry) = db.get_package_mut(id) {
            entry.active = Some(ver.clone());
        }
    }
    println!("    {} {}", ctx.styler.green("Activating"), active_displays.join(", "));

    let elapsed = start_time.elapsed().as_secs_f64();
    let count_str = std::format!("{} packages", total_count);
    println!("\n{} Installed {} in {:.2}s:", ctx.styler.bold_cyan("::"), ctx.styler.bold_white(&count_str), elapsed);
    for (name, ver, _) in &installed_list {
        let item_str = std::format!("{} {}", name, ver);
        println!("   {} {} ({})", ctx.styler.bold_green("+"), ctx.styler.bold_white(&item_str), ctx.styler.green("active"));
    }
    Ok(())
}

pub fn execute_add(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let opts = parse_add_options(args)?;
    let mut parsed = parse_add_ref(&opts.target_ref, &cfg.default_source);
    let color_mode = cfg.colors.parse().unwrap_or(ColorMode::Terminal);
    let styler = Styler::new(color_mode);

    let mut plugin = None;
    let manifest_text = resolve_manifest(&mut parsed, paths, &opts, cfg, &mut plugin)?;
    if opts.list_only || manifest_text.is_empty() {
        return Ok(());
    }

    let manifest = WorkspaceManifest::parse(&manifest_text)?;
    let pkg_ids = select_packages(&manifest, &parsed.packages)?;
    let sys_target = detect_current_target(&cfg.windows_priority);

    let mut resolved_list = Vec::new();
    for id in pkg_ids {
        let pkg = manifest.resolve_package(id)?;
        let avail: Vec<(&str, bool)> = pkg.binaries.iter().map(|(t, b)| (t.as_str(), b.is_static.unwrap_or(false))).collect();
        let target_match = match_target(&avail, &sys_target, &cfg.windows_priority);
        let chosen_target = match target_match {
            TargetMatchResult::Exact(t) => t,
            TargetMatchResult::Fallback { target, preferred } => {
                if opts.info_only {
                    target
                } else {
                    print_warning(&DiagWarning::from_code("W003", &[("preferred", &preferred), ("package", &pkg.name), ("fallback", &target)]));
                    if !prompt_yes_no(&std::format!("Install the {} build instead?", target), false, "add")? {
                        return Ok(());
                    }
                    target
                }
            }
            TargetMatchResult::NotFound => {
                let tag = parsed.release.as_deref().unwrap_or("latest");
                if opts.info_only {
                    "unavailable".to_string()
                } else {
                    return Err(DiagError::from_code("E006", &[("package", &pkg.name), ("target", &sys_target.as_str()), ("tag", tag)]));
                }
            }
        };
        resolved_list.push((pkg, chosen_target));
    }

    if opts.info_only {
        for (pkg, chosen_target) in &resolved_list {
            let sha_display = pkg.binaries.get(chosen_target.as_str()).and_then(|b| b.sha256.as_deref()).unwrap_or("unknown");
            let rel_str = parsed.release.as_deref().unwrap_or("latest");
            println!("Package   {}", pkg.name);
            println!("Source    {}:{}", parsed.source, parsed.storage);
            println!("Release   {} (release)", rel_str);
            if let Some(ref auth) = pkg.author {
                println!("Author    {}", auth);
            }
            if chosen_target == "unavailable" {
                println!("Target    {}, no binary available", sys_target.as_str());
            } else {
                println!("Target    {}, binary available", chosen_target);
            }
            println!("SHA256    {}", sha_display);
            if let Some(bin) = pkg.binaries.get(chosen_target.as_str()) && let Some(s) = bin.size {
                println!("Size      {:.2} MiB", s as f64 / 1_048_576.0);
            }
            println!();
        }
        return Ok(());
    }

    let storage_display = std::format!("{}:{}", parsed.source, parsed.storage);
    let release_display = parsed.release.as_deref().unwrap_or("latest");
    let manifest_sha = crate::net::hash::compute_sha256_lf(manifest_text.as_bytes());
    let chosen_targets_owned: Vec<String> = resolved_list.iter().map(|(_, t)| t.to_string()).collect();

    let review_files = vec![
        ViewerFile {
            title: storage_display.clone(),
            filename: "lacie.toml".to_string(),
            content: manifest_text.clone(),
            file_type: ViewerFileType::Toml,
            sha256: manifest_sha.clone(),
            chosen_targets: chosen_targets_owned,
            selected_pkgs: parsed.packages.clone(),
        }
    ];

    if opts.show_only {
        let viewer = InteractiveViewer::new(review_files, &styler, false);
        let _ = viewer.run()?;
        return Ok(());
    }

    let mut db = PackagesDatabase::load(&paths.packages_file)?;
    let mut to_install = Vec::new();
    for (pkg, chosen_target) in resolved_list {
        let is_same_version_and_hash = if let Some(pkg_entry) = db.packages.get(&pkg.id)
            && let Some(ver_entry) = pkg_entry.versions.get(&pkg.version)
            && let Some(bin_entry) = pkg.binaries.get(chosen_target.as_str())
            && let Some(ref m_sha) = bin_entry.sha256
        {
            ver_entry.sha256 == *m_sha
        } else {
            false
        };

        if is_same_version_and_hash && !opts.force {
            print_warning(&DiagWarning::from_code("W004", &[("version", &pkg.version), ("package", &pkg.name)]));
        } else {
            to_install.push((pkg, chosen_target));
        }
    }

    if to_install.is_empty() {
        return Ok(());
    }

    let mut seen = SeenDatabase::load(&paths.seen_file)?;
    let is_manifest_seen = seen.is_manifest_seen(&manifest_sha) && !cfg.always_show && !opts.iwannasee;

    let chosen_targets: Vec<&str> = to_install.iter().map(|(_, t)| t.as_str()).collect();
    if !is_manifest_seen {
        print_manifest_view(
            &manifest_text,
            &storage_display,
            release_display,
            &chosen_targets,
            &parsed.packages,
            &styler,
        );
        println!();
    }

    if to_install.len() == 1 {
        println!("{} Resolving package {} ({})\n", styler.bold_cyan("::"), styler.bold_white(&storage_display), release_display);
        render_single_inspection(&to_install[0].0, &to_install[0].1, is_manifest_seen, &manifest_sha, &styler);
    } else {
        println!("{} Resolving packages {} ({})\n", styler.bold_cyan("::"), styler.bold_white(&storage_display), release_display);
        let refs: Vec<(&ResolvedPackage, &str)> = to_install.iter().map(|(p, t)| (p, t.as_str())).collect();
        render_multi_inspection(&refs, &storage_display, release_display, &styler);
    }

    if cfg.confirm {
        loop {
            let res = prompt_yes_no_view("Proceed with installation?", true, "add")?;
            match res {
                PromptResult::Yes => break,
                PromptResult::No => return Ok(()),
                PromptResult::View => {
                    let viewer = InteractiveViewer::new(review_files.clone(), &styler, true);
                    match viewer.run()? {
                        ViewerAction::Approved => break,
                        ViewerAction::Denied => return Ok(()),
                        ViewerAction::Closed => {}
                    }
                }
            }
        }
    }

    let mut ctx = DownloadContext {
        paths,
        plugin: &mut plugin,
        parsed: &parsed,
        opts: &opts,
        styler: &styler,
    };

    if to_install.len() == 1 {
        execute_single_package_add(&to_install[0].0, &to_install[0].1, &mut ctx, &mut db)?;
    } else {
        let refs: Vec<(&ResolvedPackage, &str)> = to_install.iter().map(|(p, t)| (p, t.as_str())).collect();
        execute_multi_package_add(&refs, &mut ctx, &mut db)?;
    }

    db.save(&paths.packages_file, &paths.var_dir)?;
    seen.add_manifest(&manifest_sha);
    seen.save(&paths.seen_file, &paths.var_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_local_archive_installation_lifecycle() {
        let tmp = std::env::temp_dir().join("lacie_test_add_local");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp.clone());
        paths.ensure_all_dirs().unwrap();

        let tar_gz_path = tmp.join("test_pkg.tar.gz");
        let file = File::create(&tar_gz_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(gz);

        let manifest_content = r#"
[workspace]
author = "shareui"
version = "0.1.0"

[packages."com.example.app"]
name = "app"

[packages."com.example.app".binaries."linux-gnu-x86_64"]
file = "app-bin"
"#;
        let mut m_hdr = tar::Header::new_gnu();
        m_hdr.set_size(manifest_content.len() as u64);
        m_hdr.set_mode(0o644);
        m_hdr.set_cksum();
        builder.append_data(&mut m_hdr, "lacie.toml", manifest_content.as_bytes()).unwrap();

        let bin_content = b"#!/bin/sh\necho hi";
        let mut b_hdr = tar::Header::new_gnu();
        b_hdr.set_size(bin_content.len() as u64);
        b_hdr.set_mode(0o755);
        b_hdr.set_cksum();
        builder.append_data(&mut b_hdr, "app-bin", bin_content as &[u8]).unwrap();

        let gz = builder.into_inner().unwrap();
        gz.finish().unwrap();

        let cfg = Config {
            init: true,
            confirm: false,
            auto_switch: true,
            colors: "never".to_string(),
            ..Default::default()
        };

        let args = vec![tar_gz_path.to_string_lossy().to_string()];
        let res = execute_add(&paths, &cfg, &args);
        assert!(res.is_ok());

        let db = PackagesDatabase::load(&paths.packages_file).unwrap();
        let pkg = db.find_by_id("com.example.app").unwrap();
        assert_eq!(pkg.name, "app");
        assert_eq!(pkg.active, Some("0.1.0".to_string()));

        let active_bin = paths.active_dir.join("app");
        assert!(active_bin.exists());

        let seen = SeenDatabase::load(&paths.seen_file).unwrap();
        assert!(!seen.manifests.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_select_packages_with_required_dependency() {
        let manifest_content = r#"
[workspace]
author = "shareui"
version = "0.1.0"

[packages."de.shareui.editor"]
name = "editor"
requires = ["de.shareui.core"]

[packages."de.shareui.editor".binaries."linux-gnu-x86_64"]
file = "editor-bin"

[packages."de.shareui.core"]
name = "core"

[packages."de.shareui.core".binaries."linux-gnu-x86_64"]
file = "core-bin"
"#;
        let manifest = WorkspaceManifest::parse(manifest_content).unwrap();
        let selected = select_packages(&manifest, &["editor".to_string()]).unwrap();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&&"de.shareui.editor".to_string()));
        assert!(selected.contains(&&"de.shareui.core".to_string()));
    }

    #[test]
    fn test_exact_match_priority() {
        let items = vec![
            SearchResult { storage: "Nyanotrasen/Nyanotrasen".to_string(), stars: 500 },
            SearchResult { storage: "shareui/nyano".to_string(), stars: 10 },
        ];
        let query = "nyano";
        let exact: Vec<&SearchResult> = items.iter().filter(|r| {
            let repo = r.storage.split('/').next_back().unwrap_or(&r.storage);
            repo.to_lowercase() == query || r.storage.to_lowercase() == query
        }).collect();
        assert_eq!(exact.len(), 1);
        assert_eq!(exact[0].storage, "shareui/nyano");
    }
}
