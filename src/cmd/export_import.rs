use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::{print_warning, DiagError, DiagWarning};
use crate::model::lock::{LockFile, LockPackageEntry, LockSourceEntry};
use crate::net::client::HttpClient;
use crate::state::packages::PackagesDatabase;
use crate::state::sources::SourcesDatabase;
use crate::ui::prompt::prompt_yes_no;
use std::path::PathBuf;

pub fn execute_export(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let all_versions = args.iter().any(|a| a == "--all-versions");
    let with_config = args.iter().any(|a| a == "--with-config");
    let target = args.iter().find(|a| *a == "-" || !a.starts_with('-')).map(|s| s.as_str()).unwrap_or("lacie-lock.toml");

    let sources_db = SourcesDatabase::load(&paths.sources_file)?;
    let pkgs_db = PackagesDatabase::load(&paths.packages_file)?;
    let mut lock = LockFile::default();

    for s in sources_db.sources.values() {
        if !s.builtin {
            lock.sources.push(LockSourceEntry {
                name: s.name.clone(),
                origin: s.origin.clone(),
            });
        }
    }

    for (id, pkg) in &pkgs_db.packages {
        let versions: Vec<&crate::state::packages::PackageVersionEntry> = if all_versions {
            pkg.versions.values().collect()
        } else if let Some(act) = pkg.active.as_ref().or(pkg.frozen_version.as_ref()) {
            pkg.versions.get(act.as_str()).into_iter().collect()
        } else {
            pkg.versions.values().next_back().into_iter().collect()
        };

        for v in versions {
            if v.source == "local" {
                print_warning(&DiagWarning::from_code("W007", &[("package", &pkg.name), ("path", &v.storage)]));
            }
            let pkg_ref = if v.source == "local" {
                v.storage.clone()
            } else {
                std::format!("{}:{}#{}", v.source, v.storage, pkg.name)
            };
            lock.packages.push(LockPackageEntry {
                id: id.clone(),
                package_ref: pkg_ref,
                release: v.version.clone(),
                version: v.version.clone(),
                target: v.target.clone(),
                sha256: v.sha256.clone(),
                active: pkg.active.as_deref() == Some(&v.version),
                command: if pkg.command != pkg.name { Some(pkg.command.clone()) } else { None },
                frozen: if pkg.frozen { Some(true) } else { None },
            });
        }
    }

    if with_config {
        let mut map = std::collections::HashMap::new();
        map.insert("auto_switch".to_string(), toml::Value::Boolean(cfg.auto_switch));
        map.insert("verify_sha256".to_string(), toml::Value::Boolean(cfg.verify_sha256));
        lock.config = Some(map);
    }

    let toml_output = lock.to_toml_string()?;
    if target == "-" {
        print!("{}", toml_output);
        return Ok(());
    }

    let dest = PathBuf::from(target);
    if dest.exists() && cfg.confirm {
        let q = std::format!("Overwrite {}? [y/N]:", target);
        if !prompt_yes_no(&q, false, "export")? {
            return Ok(());
        }
    }
    std::fs::write(&dest, toml_output).map_err(|e| DiagError::new("E015", e.to_string()))?;
    println!("Exported state to {}", target);
    Ok(())
}

fn read_lock_content(target: &str, client: &HttpClient) -> Result<String, DiagError> {
    if target.starts_with("http://") || target.starts_with("https://") {
        client.download_text(target)
    } else {
        let p = target.strip_prefix("file://").unwrap_or(target);
        std::fs::read_to_string(p).map_err(|e| DiagError::new("E015", e.to_string()))
    }
}

pub fn execute_import(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let target = args.iter().find(|a| !a.starts_with('-'))
        .ok_or_else(|| DiagError::from_code("E017", &[("expected", "lock file path")]))?;
    let client = HttpClient::new();
    let content = read_lock_content(target, &client)?;
    let lock = LockFile::parse(&content)?;

    println!("Importing state: {} sources, {} packages", lock.sources.len(), lock.packages.len());
    if cfg.confirm {
        let q = std::format!("Import {} packages? [Y/n]:", lock.packages.len());
        if !prompt_yes_no(&q, true, "import")? {
            return Ok(());
        }
    }

    for s in &lock.sources {
        let _ = crate::cmd::source::execute_source(paths, cfg, &["add".to_string(), s.origin.clone()]);
    }

    let mut failures = Vec::new();
    for p in &lock.packages {
        let mut add_args = vec![p.package_ref.clone(), p.release.clone()];
        add_args.push(std::format!("--set-hash={}", p.sha256));
        if let Err(e) = crate::cmd::add::execute_add(paths, cfg, &add_args) {
            failures.push(std::format!("{} {}: {}", p.id, p.version, e.message));
        }
    }

    if !failures.is_empty() {
        eprintln!("\nFailed to import {} packages:", failures.len());
        for f in &failures {
            eprintln!("  - {}", f);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_to_stdout() {
        let tmp = std::env::temp_dir().join("lacie_test_export");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let mut cfg = Config::default();
        cfg.confirm = false;

        assert!(execute_export(&paths, &cfg, &["-".to_string()]).is_ok());
    }
}
