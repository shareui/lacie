use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::locale;
use crate::model::source_toml::SourceManifest;
use crate::net::client::HttpClient;
use crate::net::hash::compute_sha256_bytes;
use crate::state::packages::PackagesDatabase;
use crate::state::seen::SeenDatabase;
use crate::state::sources::{SourceEntry, SourcesDatabase};
use crate::ui::color::{ColorMode, Styler};
use crate::ui::diff::generate_unified_diff;
use crate::ui::prompt::prompt_yes_no;
use crate::ui::syntax::highlight_toml;
use chrono::Utc;
use std::path::{Path, PathBuf};

fn fetch_source_text(origin: &str, client: &HttpClient) -> Result<String, DiagError> {
    if origin.starts_with("http://") || origin.starts_with("https://") {
        client.download_text(origin)
    } else {
        let path_str = origin.strip_prefix("file://").unwrap_or(origin);
        std::fs::read_to_string(path_str).map_err(|e| DiagError::new("E015", e.to_string()))
    }
}

fn stage_source_files(
    ref_str: &str,
    manifest: &SourceManifest,
    staging_dir: &Path,
    client: &HttpClient,
) -> Result<(), DiagError> {
    std::fs::create_dir_all(staging_dir).map_err(|e| DiagError::new("E015", e.to_string()))?;
    if ref_str.starts_with("http://") || ref_str.starts_with("https://") {
        let base_url = match ref_str.rfind('/') {
            Some(idx) => &ref_str[..idx + 1],
            None => ref_str,
        };
        for f in &manifest.files {
            let file_url = std::format!("{}{}", base_url, f.path);
            let dest = staging_dir.join(&f.path);
            if let Some(parent) = dest.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            client.download_file(&file_url, &dest)?;
        }
    } else {
        let base_path = PathBuf::from(ref_str.strip_prefix("file://").unwrap_or(ref_str));
        let src_dir = if base_path.is_file() {
            base_path.parent().unwrap_or(Path::new("."))
        } else {
            &base_path
        };
        for f in &manifest.files {
            let src = src_dir.join(&f.path);
            let dest = staging_dir.join(&f.path);
            if let Some(parent) = dest.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if src.is_file() {
                std::fs::copy(&src, &dest).map_err(|e| DiagError::new("E015", e.to_string()))?;
            }
        }
    }
    Ok(())
}

fn source_add(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.is_empty() {
        return Err(DiagError::from_code("E017", &[("expected", "source reference")]));
    }
    let ref_str = &args[0];
    let client = HttpClient::new();
    let toml_text = fetch_source_text(ref_str, &client)?;
    let manifest = SourceManifest::parse(&toml_text)?;
    let mut sources_db = SourcesDatabase::load(&paths.sources_file)?;
    if sources_db.get(&manifest.name).is_some() {
        return Err(DiagError::from_code("E030", &[("name", &manifest.name)]));
    }

    let staging = paths.var_dir.join("staging_source").join(&manifest.name);
    let _ = std::fs::remove_dir_all(&staging);
    stage_source_files(ref_str, &manifest, &staging, &client)?;
    std::fs::write(staging.join("source.toml"), &toml_text).map_err(|e| DiagError::new("E015", e.to_string()))?;

    let styler = Styler::new(cfg.colors.parse().unwrap_or(ColorMode::Terminal));
    if cfg.confirm {
        println!("{}", highlight_toml(&toml_text, "all", &styler));
        let q = locale::t("cmd_source.add_prompt");
        if !prompt_yes_no(&q, false, "source")? {
            let _ = std::fs::remove_dir_all(&staging);
            return Ok(());
        }
    }

    let target_dir = paths.plugins_dir.join(&manifest.name);
    let _ = std::fs::remove_dir_all(&target_dir);
    std::fs::create_dir_all(&paths.plugins_dir).map_err(|e| DiagError::new("E015", e.to_string()))?;
    std::fs::rename(&staging, &target_dir).map_err(|e| DiagError::new("E015", e.to_string()))?;

    let mut seen_db = SeenDatabase::load(&paths.seen_file)?;
    seen_db.add_manifest(&compute_sha256_bytes(toml_text.as_bytes()));
    let entry = SourceEntry {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        version_number: manifest.version_number,
        origin: ref_str.to_string(),
        builtin: false,
        enabled: true,
        added_at: Some(Utc::now()),
    };
    sources_db.add(entry)?;
    sources_db.save(&paths.sources_file, &paths.var_dir)?;
    seen_db.save(&paths.seen_file, &paths.var_dir)?;

    println!("{} Installed source {} {}", styler.bold_cyan("::"), styler.bold_white(&manifest.name), styler.bold_green(&manifest.version));
    Ok(())
}

fn source_remove(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.is_empty() {
        return Err(DiagError::from_code("E017", &[("expected", "source name")]));
    }
    let mut sources_db = SourcesDatabase::load(&paths.sources_file)?;
    let name = sources_db.find_by_name_or_id(&args[0])?;
    let packages_db = PackagesDatabase::load(&paths.packages_file)?;
    let count = packages_db.packages.values()
        .filter(|p| p.versions.values().any(|v| v.source == name))
        .count();

    if cfg.confirm {
        let count_str = count.to_string();
        let q = locale::format("cmd_source.remove_prompt", &[("source", &name), ("count", &count_str)]);
        if !prompt_yes_no(&q, false, "source")? {
            return Ok(());
        }
    }
    let plugin_dir = paths.plugins_dir.join(&name);
    if plugin_dir.is_dir() {
        let _ = std::fs::remove_dir_all(plugin_dir);
    }
    sources_db.remove(&name, &cfg.default_source)?;
    sources_db.save(&paths.sources_file, &paths.var_dir)?;
    println!("{}", locale::format("cmd_source.removed", &[("name", &name)]));
    Ok(())
}

fn source_disable(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.is_empty() {
        return Err(DiagError::from_code("E017", &[("expected", "source name")]));
    }
    let mut sources_db = SourcesDatabase::load(&paths.sources_file)?;
    let name = sources_db.find_by_name_or_id(&args[0])?;
    sources_db.disable(&name, &cfg.default_source)?;
    sources_db.save(&paths.sources_file, &paths.var_dir)?;
    println!("{}", locale::format("cmd_source.disabled", &[("name", &name)]));
    Ok(())
}

fn source_enable(paths: &LaciePaths, _cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.is_empty() {
        return Err(DiagError::from_code("E017", &[("expected", "source name")]));
    }
    let mut sources_db = SourcesDatabase::load(&paths.sources_file)?;
    let name = sources_db.find_by_name_or_id(&args[0])?;
    sources_db.enable(&name)?;
    sources_db.save(&paths.sources_file, &paths.var_dir)?;
    println!("{}", locale::format("cmd_source.enabled", &[("name", &name)]));
    Ok(())
}

fn source_list(paths: &LaciePaths, args: &[String]) -> Result<(), DiagError> {
    let sources_db = SourcesDatabase::load(&paths.sources_file)?;
    if args.iter().any(|a| a == "--json") {
        let list: Vec<serde_json::Value> = sources_db.sources.values().map(|s| {
            serde_json::json!({
                "name": s.name,
                "id": s.id,
                "version": s.version,
                "status": if s.enabled { "enabled" } else { "disabled" },
                "builtin": s.builtin,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&list).unwrap_or_default());
        return Ok(());
    }
    if sources_db.sources.is_empty() {
        println!("{}", locale::t("cmd_source.no_sources"));
        return Ok(());
    }
    println!();
    for s in sources_db.sources.values() {
        let status = if s.builtin {
            "enabled (built-in)"
        } else if s.enabled {
            "enabled"
        } else {
            "disabled"
        };
        println!("{:<8} {:<6} {:<18} {}", s.name, s.version, status, s.id);
    }
    Ok(())
}

fn print_single_source_info(entry: &SourceEntry, paths: &LaciePaths, pkgs_db: &PackagesDatabase) {
    let manifest_file = paths.plugins_dir.join(&entry.name).join("source.toml");
    let manifest = std::fs::read_to_string(&manifest_file)
        .ok()
        .and_then(|t| SourceManifest::parse(&t).ok());
    let pkg_count = pkgs_db.packages.values()
        .filter(|p| p.versions.values().any(|v| v.source == entry.name))
        .count();

    println!("{} ({})\n", entry.name, entry.id);
    if let Some(ref m) = manifest {
        if let Some(ref auth) = m.author {
            println!("  Author       {}", auth);
        }
        println!("  Version      {} (number {})", m.version, m.version_number);
        if let Some(ref sc) = m.source_code {
            println!("  Source code  {}", sc);
        }
        if !m.links.is_empty() {
            println!("  Links");
            for link in &m.links {
                println!("      - {}", link);
            }
        }
        println!("\n  API          {}", m.api);
        if !m.capabilities.is_empty() {
            println!("  Capabilities {}", m.capabilities.join(", "));
        }
        println!("  Files        {} (entry: {})", m.files.len(), m.entry);
    }
    let status_str = if entry.enabled { "enabled" } else { "disabled" };
    println!("  Status       {}", status_str);
    println!("  Origin       {}", entry.origin);
    println!("  Packages     {} installed from this source", pkg_count);
}

fn source_info(paths: &LaciePaths, args: &[String]) -> Result<(), DiagError> {
    let sources_db = SourcesDatabase::load(&paths.sources_file)?;
    let pkgs_db = PackagesDatabase::load(&paths.packages_file)?;
    let target_names: Vec<String> = if !args.is_empty() && !args[0].starts_with('-') {
        vec![sources_db.find_by_name_or_id(&args[0])?]
    } else {
        sources_db.sources.keys().cloned().collect()
    };
    for (i, name) in target_names.iter().enumerate() {
        if let Some(entry) = sources_db.get(name) {
            print_single_source_info(entry, paths, &pkgs_db);
            if i + 1 < target_names.len() {
                println!();
            }
        }
    }
    Ok(())
}

struct SourceUpdateCandidate {
    pub name: String,
    pub cur_ver: String,
    pub new_ver: String,
    pub manifest: SourceManifest,
    pub toml_text: String,
}

fn check_source_updates(
    sources_db: &SourcesDatabase,
    target: Option<&str>,
    client: &HttpClient,
) -> (Vec<SourceUpdateCandidate>, Vec<String>) {
    let mut updates = Vec::new();
    let mut builtins = Vec::new();
    for s in sources_db.sources.values() {
        if let Some(t) = target && s.name != t && s.id != t {
            continue;
        }
        if s.builtin {
            builtins.push(s.name.clone());
            continue;
        }
        if let Ok(toml_text) = fetch_source_text(&s.origin, client)
            && let Ok(manifest) = SourceManifest::parse(&toml_text)
            && manifest.version_number > s.version_number
        {
            updates.push(SourceUpdateCandidate {
                name: s.name.clone(),
                cur_ver: s.version.clone(),
                new_ver: manifest.version.clone(),
                manifest,
                toml_text,
            });
        }
    }
    (updates, builtins)
}

fn apply_source_update(
    paths: &LaciePaths,
    cand: &SourceUpdateCandidate,
    client: &HttpClient,
    styler: &Styler,
) -> Result<(), DiagError> {
    let target_dir = paths.plugins_dir.join(&cand.name);
    let old_toml = std::fs::read_to_string(target_dir.join("source.toml")).unwrap_or_default();
    println!("{}", generate_unified_diff(&old_toml, &cand.toml_text, styler));

    let staging = paths.var_dir.join("staging_update").join(&cand.name);
    let _ = std::fs::remove_dir_all(&staging);
    stage_source_files(&cand.name, &cand.manifest, &staging, client)?;
    std::fs::write(staging.join("source.toml"), &cand.toml_text).map_err(|e| DiagError::new("E015", e.to_string()))?;
    let _ = std::fs::remove_dir_all(&target_dir);
    std::fs::rename(&staging, &target_dir).map_err(|e| DiagError::new("E015", e.to_string()))
}

fn source_update(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let client = HttpClient::new();
    let styler = Styler::new(cfg.colors.parse().unwrap_or(ColorMode::Terminal));
    let mut sources_db = SourcesDatabase::load(&paths.sources_file)?;
    let target = args.first().filter(|a| !a.starts_with('-') && *a != "ALL").map(|s| s.as_str());
    let (updates, builtins) = check_source_updates(&sources_db, target, &client);

    for b in &builtins {
        println!("{}", locale::format("cmd_source.builtin_note", &[("name", b)]));
    }
    if updates.is_empty() {
        println!("{}", locale::t("cmd_source.up_to_date"));
        return Ok(());
    }

    println!("\nUpdates available ({}):\n", updates.len());
    println!("  {:<8} {:<8} NEW", "SOURCE", "CURRENT");
    for u in &updates {
        println!("  {:<8} {:<8} {}", u.name, u.cur_ver, u.new_ver);
    }
    println!();
    let prompt_key = if updates.len() == 1 { "cmd_source.update_prompt" } else { "cmd_source.update_prompt_plural" };
    let q = locale::format(prompt_key, &[("count", &updates.len().to_string())]);
    if !prompt_yes_no(&q, true, "source")? {
        return Ok(());
    }

    for u in &updates {
        apply_source_update(paths, u, &client, &styler)?;
        if let Some(entry) = sources_db.sources.get_mut(&u.name) {
            entry.version = u.new_ver.clone();
            entry.version_number = u.manifest.version_number;
        }
    }
    sources_db.save(&paths.sources_file, &paths.var_dir)
}

pub fn execute_source(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.is_empty() {
        return Err(DiagError::from_code("E017", &[("expected", "subcommand")]));
    }
    let sub = &args[0];
    let subargs = &args[1..];
    match sub.as_str() {
        "add" => source_add(paths, cfg, subargs),
        "remove" => source_remove(paths, cfg, subargs),
        "disable" => source_disable(paths, cfg, subargs),
        "enable" => source_enable(paths, cfg, subargs),
        "list" => source_list(paths, subargs),
        "info" => source_info(paths, subargs),
        "update" => source_update(paths, cfg, subargs),
        _ => Err(DiagError::from_code("E017", &[("expected", "add, remove, disable, enable, update, list, info")])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_lifecycle() {
        let tmp = std::env::temp_dir().join("lacie_test_source_lifecycle");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let mut cfg = Config::default();
        cfg.confirm = false;

        let source_dir = paths.var_dir.join("fixture_src");
        std::fs::create_dir_all(&source_dir).unwrap();
        let toml_file = source_dir.join("source.toml");
        std::fs::write(&toml_file, r#"
name = "custom"
id = "org.custom.source"
version = "1.0.0"
version_number = 1
api = 1
entry = "custom.rb"

[[files]]
path = "custom.rb"
"#).unwrap();
        std::fs::write(source_dir.join("custom.rb"), "# ruby").unwrap();

        execute_source(&paths, &cfg, &["add".to_string(), toml_file.to_string_lossy().to_string()]).unwrap();
        let db = SourcesDatabase::load(&paths.sources_file).unwrap();
        assert!(db.get("custom").is_some());

        execute_source(&paths, &cfg, &["disable".to_string(), "custom".to_string()]).unwrap();
        let db2 = SourcesDatabase::load(&paths.sources_file).unwrap();
        assert!(!db2.is_enabled("custom"));

        execute_source(&paths, &cfg, &["enable".to_string(), "custom".to_string()]).unwrap();
        let db3 = SourcesDatabase::load(&paths.sources_file).unwrap();
        assert!(db3.is_enabled("custom"));

        execute_source(&paths, &cfg, &["remove".to_string(), "custom".to_string()]).unwrap();
        let db4 = SourcesDatabase::load(&paths.sources_file).unwrap();
        assert!(db4.get("custom").is_none());
    }
}
