use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::locale;
use crate::platform::fs::{activate_binary, deactivate_binary};
use crate::state::packages::PackagesDatabase;

fn resolve_target_ids(query: &str, db: &PackagesDatabase) -> Result<Vec<String>, DiagError> {
    if query == "ALL" {
        return Ok(db.packages.keys().cloned().collect());
    }
    let matches = db.find_by_name(query);
    if matches.len() > 1 {
        let ids: Vec<String> = matches.iter().map(|(id, _)| (*id).clone()).collect();
        return Err(DiagError::from_code("E012", &[("name", query), ("ids", &ids.join(", "))]));
    }
    if let Some((id, _)) = matches.first() {
        return Ok(vec![(*id).clone()]);
    }
    if db.find_by_id(query).is_some() {
        return Ok(vec![query.to_string()]);
    }
    Err(DiagError::from_code("E009", &[("name", query)]))
}

fn freeze_single_package(
    id: &str,
    db: &mut PackagesDatabase,
    paths: &LaciePaths,
    is_all: bool,
) -> Result<Option<(String, String)>, DiagError> {
    let pkg = match db.get_package_mut(id) {
        Some(p) => p,
        None => return Ok(None),
    };
    if pkg.frozen {
        if !is_all {
            println!("{}", locale::format("cmd_freeze.already_frozen", &[("name", &pkg.name)]));
        }
        return Ok(None);
    }
    let active_ver = match pkg.active.take() {
        Some(v) => v,
        None => {
            if !is_all {
                println!("{}", locale::format("cmd_freeze.no_active_version", &[("name", &pkg.name)]));
            }
            return Ok(None);
        }
    };
    let ver_entry = match pkg.versions.get(&active_ver) {
        Some(e) => e,
        None => return Ok(None),
    };
    let bin_name = std::format!("{}-{}", pkg.name, ver_entry.sha256);
    let bin_path = paths.bin_dir.join(id).join(&active_ver).join(&bin_name);
    let active_path = paths.active_dir.join(&pkg.command);
    deactivate_binary(&active_path, &bin_path).map_err(|e| DiagError::new("E015", e.to_string()))?;
    pkg.frozen = true;
    pkg.frozen_version = Some(active_ver.clone());
    Ok(Some((pkg.name.clone(), active_ver)))
}

pub fn execute_freeze(paths: &LaciePaths, _cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.len() != 1 {
        return Err(DiagError::from_code("E017", &[("expected", "package name or ALL")]));
    }
    let query = &args[0];
    let is_all = query == "ALL";
    let mut db = PackagesDatabase::load(&paths.packages_file)?;
    let target_ids = resolve_target_ids(query, &db)?;

    for id in &target_ids {
        if let Some((name, ver)) = freeze_single_package(id, &mut db, paths, is_all)? {
            println!("{}", locale::format("cmd_freeze.frozen", &[("name", &name), ("version", &ver)]));
        }
    }
    db.save(&paths.packages_file, &paths.var_dir)
}

fn unfreeze_single_package(
    id: &str,
    db: &mut PackagesDatabase,
    paths: &LaciePaths,
    is_all: bool,
) -> Result<Option<(String, String)>, DiagError> {
    let pkg = match db.get_package_mut(id) {
        Some(p) => p,
        None => return Ok(None),
    };
    if !pkg.frozen {
        if !is_all {
            println!("{}", locale::format("cmd_freeze.not_frozen", &[("name", &pkg.name)]));
        }
        return Ok(None);
    }
    let target_ver = match pkg.frozen_version.take() {
        Some(v) => v,
        None => match pkg.versions.keys().next_back() {
            Some(v) => v.clone(),
            None => return Ok(None),
        },
    };
    let ver_entry = match pkg.versions.get(&target_ver) {
        Some(e) => e,
        None => return Ok(None),
    };
    let bin_name = std::format!("{}-{}", pkg.name, ver_entry.sha256);
    let bin_path = paths.bin_dir.join(id).join(&target_ver).join(&bin_name);
    let active_path = paths.active_dir.join(&pkg.command);
    activate_binary(&bin_path, &active_path).map_err(|e| DiagError::new("E015", e.to_string()))?;
    pkg.frozen = false;
    pkg.active = Some(target_ver.clone());
    Ok(Some((pkg.name.clone(), target_ver)))
}

pub fn execute_unfreeze(paths: &LaciePaths, _cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.len() != 1 {
        return Err(DiagError::from_code("E017", &[("expected", "package name or ALL")]));
    }
    let query = &args[0];
    let is_all = query == "ALL";
    let mut db = PackagesDatabase::load(&paths.packages_file)?;
    let target_ids = resolve_target_ids(query, &db)?;

    for id in &target_ids {
        if let Some((name, ver)) = unfreeze_single_package(id, &mut db, paths, is_all)? {
            println!("{}", locale::format("cmd_freeze.unfrozen", &[("name", &name), ("version", &ver)]));
        }
    }
    db.save(&paths.packages_file, &paths.var_dir)
}

pub fn execute_frozen(paths: &LaciePaths, _cfg: &Config, _args: &[String]) -> Result<(), DiagError> {
    let db = PackagesDatabase::load(&paths.packages_file)?;
    let mut frozen_pkgs: Vec<(&String, &String, &str)> = Vec::new();
    for (id, pkg) in &db.packages {
        if pkg.frozen {
            let ver = pkg.frozen_version.as_deref().unwrap_or("-");
            frozen_pkgs.push((&pkg.name, id, ver));
        }
    }
    if frozen_pkgs.is_empty() {
        println!("{}", locale::t("cmd_freeze.no_frozen_packages"));
        return Ok(());
    }
    frozen_pkgs.sort_by_key(|(name, _, _)| *name);
    let name_width = frozen_pkgs.iter().map(|(n, _, _)| n.len()).max().unwrap_or(4);
    let ver_width = frozen_pkgs.iter().map(|(_, _, v)| v.len()).max().unwrap_or(5);

    println!();
    for (name, id, ver) in frozen_pkgs {
        println!("{:<w1$}  {:<w2$}  {}", name, ver, id, w1 = name_width, w2 = ver_width);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::packages::PackageVersionEntry;

    #[test]
    fn test_freeze_and_unfreeze_lifecycle() {
        let tmp = std::env::temp_dir().join("lacie_test_freeze");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let cfg = Config::default();

        let mut db = PackagesDatabase::default();
        let bin_dir = paths.bin_dir.join("com.example.tool").join("1.0.0");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let bin_file = bin_dir.join("tool-sha123");
        std::fs::write(&bin_file, "binary").unwrap();
        let active_symlink = paths.active_dir.join("tool");
        activate_binary(&bin_file, &active_symlink).unwrap();

        db.insert_version("com.example.tool", "tool", Some("1.0.0"), PackageVersionEntry {
            version: "1.0.0".to_string(),
            version_number: 1,
            author: None,
            links: vec![],
            source_code: None,
            installed_at: None,
            released_at: None,
            source: "builtin".to_string(),
            storage: "repo".to_string(),
            target: "x86_64".to_string(),
            sha256: "sha123".to_string(),
        });
        if let Some(pkg) = db.get_package_mut("com.example.tool") {
            pkg.active = Some("1.0.0".to_string());
        }
        db.save(&paths.packages_file, &paths.var_dir).unwrap();

        execute_freeze(&paths, &cfg, &["tool".to_string()]).unwrap();
        let db_after = PackagesDatabase::load(&paths.packages_file).unwrap();
        let pkg = db_after.find_by_id("com.example.tool").unwrap();
        assert!(pkg.frozen);
        assert_eq!(pkg.frozen_version.as_deref(), Some("1.0.0"));
        assert_eq!(pkg.active, None);

        execute_unfreeze(&paths, &cfg, &["tool".to_string()]).unwrap();
        let db_unfrozen = PackagesDatabase::load(&paths.packages_file).unwrap();
        let pkg2 = db_unfrozen.find_by_id("com.example.tool").unwrap();
        assert!(!pkg2.frozen);
        assert_eq!(pkg2.active.as_deref(), Some("1.0.0"));
    }
}
