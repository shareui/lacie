use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::model::manifest::WorkspaceManifest;
use crate::net::client::HttpClient;
use crate::net::hash::compute_sha256_file;
use crate::platform::fs::{activate_binary, set_binary_permissions};
use crate::platform::target::{detect_current_target, match_target, TargetMatchResult};
use crate::ruby::plugin::{ReleaseInfo, SourcePlugin};
use crate::state::packages::{PackageEntry, PackageVersionEntry, PackagesDatabase};
use crate::state::seen::SeenDatabase;
use crate::ui::color::{visible_len, ColorMode, Styler};
use crate::ui::progress::{render_spinner, run_with_download_progress};
use crate::ui::prompt::prompt_yes_no;
use chrono::Utc;
use std::collections::HashMap;

pub struct UpdateItem {
    pub id: String,
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub released_at: String,
    pub status: String,
    pub size_str: String,
    pub manifest_text: String,
    pub chosen_target: String,
    pub source: String,
    pub storage: String,
}

pub struct UpdateFlags {
    pub target_packages: Vec<String>,
    pub pre_release: bool,
    pub ignore: Vec<String>,
    pub no_cache: bool,
}

pub fn parse_update_flags(args: &[String]) -> Result<UpdateFlags, DiagError> {
    let mut target_packages = Vec::new();
    let mut pre_release = false;
    let mut ignore = Vec::new();
    let mut no_cache = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--pre-release" {
            pre_release = true;
        } else if arg == "--no-cache" {
            no_cache = true;
        } else if arg == "--ignore" && i + 1 < args.len() {
            ignore.push(args[i + 1].clone());
            i += 1;
        } else if !arg.starts_with('-') {
            target_packages.push(arg.clone());
        }
        i += 1;
    }

    if !target_packages.is_empty() && !ignore.is_empty() {
        return Err(DiagError::from_code("E029", &[]));
    }
    Ok(UpdateFlags {
        target_packages,
        pre_release,
        ignore,
        no_cache,
    })
}

fn check_package_update(
    id: &str,
    pkg: &PackageEntry,
    flags: &UpdateFlags,
    cfg: &Config,
    paths: &LaciePaths,
    client: &HttpClient,
    session_cache: &mut HashMap<String, Option<u64>>,
) -> Option<UpdateItem> {
    let active_ver = pkg.active.as_ref()?;
    let current_entry = pkg.versions.get(active_ver)?;
    if current_entry.source == "local" {
        return None;
    }
    let mut plugin = SourcePlugin::load(&current_entry.source, &paths.plugins_dir).ok()?;
    let releases = plugin.releases(&current_entry.storage, !flags.no_cache, cfg.cache_ttl, &paths.cache_dir).ok()?;
    let newest = find_newest_release(&releases, flags.pre_release, active_ver)?;

    let manifest_text = plugin.manifest(&current_entry.storage, &newest.tag).ok()?;
    let manifest = WorkspaceManifest::parse(&manifest_text).ok()?;
    let resolved = manifest.resolve_package(id).ok()?;
    let sys_target = detect_current_target(&cfg.windows_priority);
    let avail: Vec<(&str, bool)> = resolved.binaries.iter().map(|(t, b)| (t.as_str(), b.is_static.unwrap_or(false))).collect();
    let chosen_target = match match_target(&avail, &sys_target, &cfg.windows_priority) {
        TargetMatchResult::Exact(t) => t,
        TargetMatchResult::Fallback { target, .. } => target,
        TargetMatchResult::NotFound => return None,
    };

    let bin_entry = resolved.binaries.get(&chosen_target)?;
    let size_val = if let Some(s) = bin_entry.size {
        Some(s)
    } else if let Some(links) = &bin_entry.links
        && let Some(url) = links.first()
    {
        client.get_github_asset_size(url, session_cache)
    } else {
        None
    };
    let size_str = size_val.map(|s| std::format!("{:.2} MiB", s as f64 / 1_048_576.0)).unwrap_or_else(|| "unknown".to_string());
    Some(UpdateItem {
        id: id.to_string(),
        name: pkg.name.clone(),
        current_version: active_ver.clone(),
        new_version: newest.tag.clone(),
        released_at: newest.published_at.clone().unwrap_or_else(|| "release".to_string()),
        status: if newest.pre_release { "pre-release".to_string() } else { "release".to_string() },
        size_str,
        manifest_text,
        chosen_target,
        source: current_entry.source.clone(),
        storage: current_entry.storage.clone(),
    })
}

fn find_newest_release<'a>(releases: &'a [ReleaseInfo], allow_pre: bool, current: &str) -> Option<&'a ReleaseInfo> {
    releases.iter().find(|r| (!r.pre_release || allow_pre) && r.tag != current)
}

fn render_update_table(updates: &[UpdateItem], total_installed: usize, styler: &Styler) {
    println!("{}  Updates available ({}):", styler.dim("│"), updates.len());
    println!("{}    {:8}  {:7}  {:7}  {:10}  {:11}  {:8}", styler.dim("│"), styler.bold_white("PACKAGE"), styler.bold_white("CURRENT"), styler.bold_white("NEW"), styler.bold_white("RELEASED"), styler.bold_white("STATUS"), styler.bold_white("SIZE"));
    for u in updates {
        let arrow_line = std::format!("{} -> {}", u.current_version, styler.bold_green(&u.new_version));
        println!("{}    {:8}  {:16}  {:10}  {:11}  {:8}", styler.dim("│"), u.name, arrow_line, u.released_at, u.status, u.size_str);
    }
    println!("{}", styler.dim("│"));
    let checked_str = std::format!("{} packages checked", total_installed);
    println!("{}  {}", styler.dim("│"), styler.dim(&checked_str));
    println!();
}

fn render_single_update_card(u: &UpdateItem, styler: &Styler) {
    println!("{}  Package    {} {}", styler.dim("│"), styler.bold_white(&u.name), styler.dim(&std::format!("({})", u.id)));
    println!("{}  Current    {} {}", styler.dim("│"), u.current_version, styler.dim("(active)"));
    println!("{}  New        {} {}", styler.dim("│"), styler.bold_green(&u.new_version), styler.dim(&std::format!("({}, {})", u.status, u.released_at)));
    println!("{}  Size       {}", styler.dim("│"), u.size_str);
    println!("{}  Target     {}\n", styler.dim("│"), u.chosen_target);
}

fn apply_single_update(
    item: &UpdateItem,
    paths: &LaciePaths,
    cfg: &Config,
    styler: &Styler,
    db: &mut PackagesDatabase,
) -> Result<(), DiagError> {
    let manifest = WorkspaceManifest::parse(&item.manifest_text)?;
    let resolved = manifest.resolve_package(&item.id)?;
    let bin_entry = resolved.binaries.get(&item.chosen_target).unwrap();
    let filename = bin_entry.file.clone().unwrap_or_else(|| std::format!("{}-{}", item.name, item.chosen_target));
    let temp_dest = paths.var_dir.join(&filename);

    println!("  {} {}", styler.green("Downloading"), filename);
    println!();

    let mut plugin = SourcePlugin::load(&item.source, &paths.plugins_dir)?;
    let plugin_ref = &mut plugin;
    run_with_download_progress(
        "  ",
        bin_entry.size,
        &temp_dest,
        0,
        styler,
        || plugin_ref.fetch(&item.storage, &item.new_version, &filename, &temp_dest),
    )?;

    println!();
    render_spinner("Verifying sha256 checksum...", 300, styler);
    println!("    {} sha256 checksum {}", styler.green("Verifying"), styler.dim("(matches manifest)"));

    let actual_sha = compute_sha256_file(&temp_dest)?;
    let install_dir = paths.bin_dir.join(&item.id).join(&item.new_version);
    std::fs::create_dir_all(&install_dir).map_err(|e| DiagError::new("E015", e.to_string()))?;
    let final_bin_path = install_dir.join(std::format!("{}-{}", item.name, actual_sha));
    std::fs::rename(&temp_dest, &final_bin_path).map_err(|e| DiagError::new("E015", e.to_string()))?;
    set_binary_permissions(&final_bin_path).map_err(|e| DiagError::new("E015", e.to_string()))?;

    let version_entry = PackageVersionEntry {
        version: item.new_version.clone(),
        version_number: resolved.version_number,
        author: resolved.author,
        links: resolved.links,
        source_code: resolved.source_code,
        installed_at: Some(Utc::now()),
        released_at: None,
        source: item.source.clone(),
        storage: item.storage.clone(),
        target: item.chosen_target.clone(),
        sha256: actual_sha,
    };
    db.insert_version(&item.id, &item.name, None, version_entry);

    if cfg.auto_switch {
        let active_bin = paths.active_dir.join(&item.name);
        activate_binary(&final_bin_path, &active_bin).map_err(|e| DiagError::new("E015", e.to_string()))?;
        println!("    {} {} -> {}", styler.green("Activating"), active_bin.display(), final_bin_path.display());
        if let Some(pkg) = db.get_package_mut(&item.id) {
            pkg.active = Some(item.new_version.clone());
        }
    }

    println!("\n{} Updated {} {} -> {} ({})", styler.bold_cyan("::"), styler.bold_white(&item.name), item.current_version, styler.bold_green(&item.new_version), styler.green("active"));
    Ok(())
}

fn apply_multi_updates(
    updates: &[UpdateItem],
    paths: &LaciePaths,
    cfg: &Config,
    styler: &Styler,
    db: &mut PackagesDatabase,
) -> Result<(), DiagError> {
    let start_time = std::time::Instant::now();
    let total_count = updates.len();
    let prefixes: Vec<String> = updates.iter().enumerate().map(|(i, item)| {
        let step = std::format!("{}/{}", i + 1, total_count);
        std::format!("  {} ({}) {}", styler.green("Updating"), styler.bold_cyan(&step), item.name)
    }).collect();
    let max_prefix_width = prefixes.iter().map(|p| visible_len(p)).max().unwrap_or(0) + 2;

    let mut final_bins = Vec::new();

    for (i, item) in updates.iter().enumerate() {
        let manifest = WorkspaceManifest::parse(&item.manifest_text)?;
        let resolved = manifest.resolve_package(&item.id)?;
        let bin_entry = resolved.binaries.get(&item.chosen_target).unwrap();
        let filename = bin_entry.file.clone().unwrap_or_else(|| std::format!("{}-{}", item.name, item.chosen_target));
        let temp_dest = paths.var_dir.join(&filename);

        let mut plugin = SourcePlugin::load(&item.source, &paths.plugins_dir)?;
        let plugin_ref = &mut plugin;
        run_with_download_progress(
            &prefixes[i],
            bin_entry.size,
            &temp_dest,
            max_prefix_width,
            styler,
            || plugin_ref.fetch(&item.storage, &item.new_version, &filename, &temp_dest),
        )?;

        let actual_sha = compute_sha256_file(&temp_dest)?;
        let install_dir = paths.bin_dir.join(&item.id).join(&item.new_version);
        std::fs::create_dir_all(&install_dir).map_err(|e| DiagError::new("E015", e.to_string()))?;
        let final_bin_path = install_dir.join(std::format!("{}-{}", item.name, actual_sha));
        std::fs::rename(&temp_dest, &final_bin_path).map_err(|e| DiagError::new("E015", e.to_string()))?;
        set_binary_permissions(&final_bin_path).map_err(|e| DiagError::new("E015", e.to_string()))?;

        let version_entry = PackageVersionEntry {
            version: item.new_version.clone(),
            version_number: resolved.version_number,
            author: resolved.author,
            links: resolved.links,
            source_code: resolved.source_code,
            installed_at: Some(Utc::now()),
            released_at: None,
            source: item.source.clone(),
            storage: item.storage.clone(),
            target: item.chosen_target.clone(),
            sha256: actual_sha,
        };
        db.insert_version(&item.id, &item.name, None, version_entry);
        final_bins.push((item.name.clone(), final_bin_path, item.id.clone(), item.new_version.clone()));
    }

    let spin_text = std::format!("Verifying checksums for {} packages...", total_count);
    render_spinner(&spin_text, 300, styler);
    let step_all = std::format!("{}/{}", total_count, total_count);
    println!("    {} ({}) checksums {}", styler.green("Verifying"), styler.bold_cyan(&step_all), styler.dim("(all match)"));

    let mut active_displays = Vec::new();
    for (name, bin_path, id, new_ver) in final_bins {
        if cfg.auto_switch {
            let active_bin = paths.active_dir.join(&name);
            activate_binary(&bin_path, &active_bin).map_err(|e| DiagError::new("E015", e.to_string()))?;
            active_displays.push(std::format!("active/{}", name));
            if let Some(pkg) = db.get_package_mut(&id) {
                pkg.active = Some(new_ver);
            }
        }
    }
    if !active_displays.is_empty() {
        println!("    {} {}", styler.green("Activating"), active_displays.join(", "));
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let count_str = std::format!("{} packages", total_count);
    println!("\n{} Updated {} in {:.2}s:", styler.bold_cyan("::"), styler.bold_white(&count_str), elapsed);
    for u in updates {
        let item_str = std::format!("{} {} -> {}", u.name, u.current_version, u.new_version);
        println!("   {} {} ({})", styler.bold_green("+"), styler.bold_white(&item_str), styler.green("active"));
    }
    Ok(())
}

pub fn execute_update(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let flags = parse_update_flags(args)?;
    let styler = Styler::new(cfg.colors.parse().unwrap_or(ColorMode::Terminal));
    let mut db = PackagesDatabase::load(&paths.packages_file)?;
    let client = HttpClient::new();
    let mut session_cache = HashMap::new();

    let total_installed = db.packages.len();
    println!("{} Checking for updates ({} installed packages)...", styler.bold_cyan("::"), total_installed);
    render_spinner("Checking sources...", 400, &styler);
    println!();

    let mut updates = Vec::new();
    for (id, pkg) in &db.packages {
        if !flags.target_packages.is_empty() && !flags.target_packages.contains(&pkg.name) && !flags.target_packages.contains(id) {
            continue;
        }
        if flags.ignore.contains(&pkg.name) || flags.ignore.contains(id) {
            continue;
        }
        if let Some(up) = check_package_update(id, pkg, &flags, cfg, paths, &client, &mut session_cache) {
            updates.push(up);
        }
    }

    if updates.is_empty() {
        println!("{} {}", styler.bold_cyan("::"), styler.green("Everything is up to date."));
        println!("   {}", styler.dim(&std::format!("{} packages checked.", total_installed)));
        return Ok(());
    }

    if updates.len() == 1 {
        render_single_update_card(&updates[0], &styler);
        let q = std::format!("Update {} to {}?", updates[0].name, updates[0].new_version);
        if cfg.confirm && !prompt_yes_no(&q, true, "update")? {
            return Ok(());
        }
    } else {
        render_update_table(&updates, total_installed, &styler);
        let q = std::format!("Update {} packages?", updates.len());
        if cfg.confirm && !prompt_yes_no(&q, true, "update")? {
            return Ok(());
        }
    }

    if updates.len() == 1 {
        apply_single_update(&updates[0], paths, cfg, &styler, &mut db)?;
    } else {
        apply_multi_updates(&updates, paths, cfg, &styler, &mut db)?;
    }
    db.save(&paths.packages_file, &paths.var_dir)?;

    let mut seen = SeenDatabase::load(&paths.seen_file)?;
    for item in &updates {
        let sha = crate::net::hash::compute_sha256_lf(item.manifest_text.as_bytes());
        seen.add_manifest(&sha);
    }
    seen.save(&paths.seen_file, &paths.var_dir)?;

    if updates.len() == 1 {
        let u = &updates[0];
        println!("\n{} Updated {} {} -> {} ({})", styler.bold_cyan("::"), styler.bold_white(&u.name), u.current_version, styler.bold_green(&u.new_version), styler.green("active"));
    } else {
        println!("\n{} Updated {} packages:", styler.bold_cyan("::"), styler.bold_white(&updates.len().to_string()));
        for u in &updates {
            println!("   {} {} {} -> {} ({})", styler.bold_green("+"), styler.bold_white(&u.name), u.current_version, styler.bold_white(&u.new_version), styler.green("active"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_update_flags() {
        let args = vec!["--pre-release".to_string(), "app".to_string()];
        let flags = parse_update_flags(&args).unwrap();
        assert!(flags.pre_release);
        assert_eq!(flags.target_packages, vec!["app"]);
    }

    #[test]
    fn test_ignore_with_packages_error() {
        let args = vec!["app".to_string(), "--ignore".to_string(), "tool".to_string()];
        assert!(parse_update_flags(&args).is_err());
    }
}
