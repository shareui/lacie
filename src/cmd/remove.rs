use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::{print_warning, DiagError, DiagWarning};
use crate::net::hash::compute_sha256_file;
use crate::ruby::runner::{execute_ruby_script, HookContext};
use crate::state::packages::PackagesDatabase;
use crate::state::scripts::ScriptsDatabase;
use crate::ui::color::{ColorMode, Styler};
use crate::ui::prompt::prompt_yes_no;

pub struct RemoveFlags {
    pub packages: Vec<String>,
    pub unused_only: bool,
    pub symlink_only: bool,
    pub active_only: bool,
    pub no_scripts: bool,
}

pub fn parse_remove_flags(args: &[String]) -> Result<RemoveFlags, DiagError> {
    let mut packages = Vec::new();
    let mut unused_only = false;
    let mut symlink_only = false;
    let mut active_only = false;
    let mut no_scripts = false;

    for arg in args {
        if arg == "--unused" {
            unused_only = true;
        } else if arg == "--symlink" {
            symlink_only = true;
        } else if arg == "--active" {
            active_only = true;
        } else if arg == "--no-scripts" {
            no_scripts = true;
        } else if !arg.starts_with('-') {
            packages.push(arg.clone());
        }
    }
    if packages.is_empty() {
        return Err(DiagError::from_code("E017", &[("expected", "package name or ALL")]));
    }
    Ok(RemoveFlags {
        packages,
        unused_only,
        symlink_only,
        active_only,
        no_scripts,
    })
}

fn execute_hook_script(
    script_path: &std::path::Path,
    hook_name: &str,
    expected_sha: &str,
    id: &str,
    version: &str,
    paths: &LaciePaths,
    is_last: bool,
) -> Result<(), DiagError> {
    if !script_path.is_file() {
        return Ok(());
    }
    let actual_sha = compute_sha256_file(script_path)?;
    if actual_sha != expected_sha {
        return Err(DiagError::from_code("E035", &[("hook", hook_name), ("package", id), ("version", version)]));
    }
    let ctx = HookContext {
        pkg_id: id,
        pkg_version: version,
        pkg_version_number: 1,
        target: "unknown",
        dest_dir: &paths.bin_dir.join(id).join(version),
        hook_name,
        is_last,
    };
    execute_ruby_script(script_path, &paths.var_dir, &ctx)
}

fn remove_package_version_files(
    id: &str,
    version: &str,
    paths: &LaciePaths,
    scripts_db: &mut ScriptsDatabase,
    no_scripts: bool,
    is_last: bool,
) -> Result<(), DiagError> {
    let script_dir = paths.scripts_dir.join(id).join(version);
    let preremove = script_dir.join("preremove.rb");
    let postremove = script_dir.join("postremove.rb");

    if !no_scripts && let Some(hashes) = scripts_db.get(id, version)
        && let Some(ref expected) = hashes.preremove
    {
        execute_hook_script(&preremove, "preremove", expected, id, version, paths, is_last)?;
    }

    let bin_version_dir = paths.bin_dir.join(id).join(version);
    if bin_version_dir.is_dir() {
        let _ = std::fs::remove_dir_all(&bin_version_dir);
    }

    if !no_scripts && let Some(hashes) = scripts_db.get(id, version)
        && let Some(ref expected) = hashes.postremove
        && let Err(e) = execute_hook_script(&postremove, "postremove", expected, id, version, paths, is_last)
    {
        print_warning(&DiagWarning::from_code("W005", &[("package", id), ("reason", &e.message)]));
    }
    let _ = std::fs::remove_dir_all(&script_dir);
    scripts_db.remove(id, version);
    Ok(())
}

pub fn execute_remove(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let flags = parse_remove_flags(args)?;
    let styler = Styler::new(cfg.colors.parse().unwrap_or(ColorMode::Terminal));
    let mut db = PackagesDatabase::load(&paths.packages_file)?;
    let mut scripts_db = ScriptsDatabase::load(&paths.scripts_file)?;

    let target_ids: Vec<String> = if flags.packages.contains(&"ALL".to_string()) {
        db.packages.keys().cloned().collect()
    } else {
        let mut ids = Vec::new();
        for query in &flags.packages {
            let matches = db.find_by_name(query);
            if matches.len() > 1 {
                let id_list: Vec<String> = matches.iter().map(|(id, _)| (*id).clone()).collect();
                return Err(DiagError::from_code("E012", &[("name", query), ("ids", &id_list.join(", "))]));
            } else if let Some((id, _)) = matches.first() {
                ids.push((*id).clone());
            } else if db.find_by_id(query).is_some() {
                ids.push(query.clone());
            } else {
                return Err(DiagError::from_code("E009", &[("name", query)]));
            }
        }
        ids
    };

    if cfg.confirm {
        let count_str = target_ids.len().to_string();
        let q = crate::locale::format("cmd_remove.remove_prompt", &[("count", &count_str)]);
        if !prompt_yes_no(&q, false, "remove")? {
            return Ok(());
        }
    }

    for id in &target_ids {
        let pkg = db.find_by_id(id).unwrap().clone();
        let active_symlink = paths.active_dir.join(&pkg.command);
        if active_symlink.exists() || active_symlink.is_symlink() {
            let _ = std::fs::remove_file(&active_symlink);
        }
        if flags.symlink_only {
            continue;
        }

        let versions_to_remove: Vec<String> = if flags.active_only {
            pkg.active.into_iter().collect()
        } else if flags.unused_only {
            pkg.versions.keys().filter(|v| Some(*v) != pkg.active.as_ref()).cloned().collect()
        } else {
            pkg.versions.keys().cloned().collect()
        };

        let total_versions = pkg.versions.len();
        for (i, ver) in versions_to_remove.iter().enumerate() {
            let is_last = total_versions == versions_to_remove.len() && i == versions_to_remove.len() - 1;
            remove_package_version_files(id, ver, paths, &mut scripts_db, flags.no_scripts, is_last)?;
            db.remove_version(id, ver);
        }
    }

    db.save(&paths.packages_file, &paths.var_dir)?;
    scripts_db.save(&paths.scripts_file, &paths.var_dir)?;
    let count_str = styler.bold_white(&target_ids.len().to_string());
    let msg = crate::locale::format("cmd_remove.removed_count", &[("count", &count_str)]);
    println!("{} {}", styler.bold_cyan("::"), msg);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::packages::PackageVersionEntry;

    #[test]
    fn test_remove_package_symlink_and_files() {
        let tmp = std::env::temp_dir().join("lacie_test_remove");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp.clone());
        paths.ensure_all_dirs().unwrap();

        let mut db = PackagesDatabase::default();
        let bin_v1 = paths.bin_dir.join("com.example.app").join("1.0.0");
        std::fs::create_dir_all(&bin_v1).unwrap();
        std::fs::write(bin_v1.join("app-sha1"), "binary").unwrap();
        let active_link = paths.active_dir.join("app");
        std::fs::write(&active_link, "link").unwrap();

        db.insert_version("com.example.app", "app", None, PackageVersionEntry {
            version: "1.0.0".to_string(),
            version_number: 1,
            author: None,
            links: vec![],
            source_code: None,
            installed_at: None,
            released_at: None,
            source: "github".to_string(),
            storage: "shareui/app".to_string(),
            target: "linux-gnu-x86_64".to_string(),
            sha256: "sha1".to_string(),
        });
        db.get_package_mut("com.example.app").unwrap().active = Some("1.0.0".to_string());
        db.save(&paths.packages_file, &paths.var_dir).unwrap();

        let cfg = Config { confirm: false, colors: "never".to_string(), ..Default::default() };
        let args = vec!["app".to_string()];
        execute_remove(&paths, &cfg, &args).unwrap();

        let reloaded = PackagesDatabase::load(&paths.packages_file).unwrap();
        assert!(reloaded.find_by_id("com.example.app").is_none());
        assert!(!active_link.exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
