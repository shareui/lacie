use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::platform::fs::activate_binary;
use crate::state::packages::PackagesDatabase;
use crate::ui::color::{ColorMode, Styler};

struct SwitchTarget {
    pub id: String,
    pub name: String,
    pub command: String,
    pub new_version: String,
    pub bin_path: std::path::PathBuf,
}

fn resolve_switch_pair(
    pkg_query: &str,
    ver_query: &str,
    db: &PackagesDatabase,
    paths: &LaciePaths,
) -> Result<SwitchTarget, DiagError> {
    let matches = db.find_by_name(pkg_query);
    let (id, pkg) = if matches.len() > 1 {
        let ids: Vec<String> = matches.iter().map(|(id, _)| (*id).clone()).collect();
        return Err(DiagError::from_code("E012", &[("name", pkg_query), ("ids", &ids.join(", "))]));
    } else if let Some((id, pkg)) = matches.first() {
        ((*id).clone(), *pkg)
    } else if let Some(pkg) = db.find_by_id(pkg_query) {
        (pkg_query.to_string(), pkg)
    } else {
        return Err(DiagError::from_code("E009", &[("name", pkg_query)]));
    };

    if pkg.frozen {
        return Err(DiagError::from_code("E020", &[("name", &pkg.name)]));
    }

    let target_version = if ver_query.to_uppercase() == "LATEST" {
        let latest = pkg.versions.values().max_by_key(|v| v.version_number)
            .ok_or_else(|| DiagError::from_code("E009", &[("name", &pkg.name)]))?;
        latest.version.clone()
    } else {
        if !pkg.versions.contains_key(ver_query) {
            let installed: Vec<String> = pkg.versions.keys().cloned().collect();
            return Err(DiagError::from_code("E010", &[("name", &pkg.name), ("version", ver_query), ("versions", &installed.join(", "))]));
        }
        ver_query.to_string()
    };

    let ver_entry = pkg.versions.get(&target_version).unwrap();
    let bin_name = std::format!("{}-{}", pkg.name, ver_entry.sha256);
    let bin_path = paths.bin_dir.join(&id).join(&target_version).join(&bin_name);
    if !bin_path.exists() {
        return Err(DiagError::from_code("E010", &[("name", &pkg.name), ("version", &target_version), ("versions", &target_version)]));
    }

    Ok(SwitchTarget {
        id,
        name: pkg.name.clone(),
        command: pkg.command.clone(),
        new_version: target_version,
        bin_path,
    })
}

pub fn execute_switch(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.is_empty() || !args.len().is_multiple_of(2) {
        return Err(DiagError::from_code("E011", &[]));
    }
    let styler = Styler::new(cfg.colors.parse().unwrap_or(ColorMode::Terminal));
    let mut db = PackagesDatabase::load(&paths.packages_file)?;

    let mut switch_targets = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let pkg_query = &args[i];
        let ver_query = &args[i + 1];
        let target = resolve_switch_pair(pkg_query, ver_query, &db, paths)?;
        switch_targets.push(target);
        i += 2;
    }

    for target in &switch_targets {
        let active_path = paths.active_dir.join(&target.command);
        activate_binary(&target.bin_path, &active_path)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        if let Some(pkg) = db.get_package_mut(&target.id) {
            pkg.active = Some(target.new_version.clone());
        }
    }
    db.save(&paths.packages_file, &paths.var_dir)?;

    if switch_targets.len() == 1 {
        let t = &switch_targets[0];
        let msg = crate::locale::format("cmd_switch.switched_single", &[("name", &styler.bold_white(&t.name)), ("version", &styler.bold_green(&t.new_version))]);
        println!("{} {}", styler.bold_cyan("::"), msg);
    } else {
        let count_str = styler.bold_white(&switch_targets.len().to_string());
        let msg = crate::locale::format("cmd_switch.switched_multi", &[("count", &count_str)]);
        println!("{} {}", styler.bold_cyan("::"), msg);
        for t in &switch_targets {
            let item_msg = crate::locale::format("cmd_switch.switched_item", &[("name", &styler.bold_white(&t.name)), ("version", &styler.bold_green(&t.new_version))]);
            println!("   {} {}", styler.bold_green("+"), item_msg);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::packages::PackageVersionEntry;

    #[test]
    fn test_switch_odd_arguments() {
        let tmp = std::env::temp_dir().join("lacie_test_switch_odd");
        let paths = LaciePaths::from_root(tmp);
        let cfg = Config::default();
        let args = vec!["app".to_string()];
        assert_eq!(execute_switch(&paths, &cfg, &args).unwrap_err().code, "E011");
    }

    #[test]
    fn test_switch_lifecycle() {
        let tmp = std::env::temp_dir().join("lacie_test_switch_ok");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp.clone());
        paths.ensure_all_dirs().unwrap();

        let mut db = PackagesDatabase::default();
        let bin_v1 = paths.bin_dir.join("com.example.app").join("1.0.0");
        let bin_v2 = paths.bin_dir.join("com.example.app").join("2.0.0");
        std::fs::create_dir_all(&bin_v1).unwrap();
        std::fs::create_dir_all(&bin_v2).unwrap();
        std::fs::write(bin_v1.join("app-sha1"), "v1").unwrap();
        std::fs::write(bin_v2.join("app-sha2"), "v2").unwrap();

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
        db.insert_version("com.example.app", "app", None, PackageVersionEntry {
            version: "2.0.0".to_string(),
            version_number: 2,
            author: None,
            links: vec![],
            source_code: None,
            installed_at: None,
            released_at: None,
            source: "github".to_string(),
            storage: "shareui/app".to_string(),
            target: "linux-gnu-x86_64".to_string(),
            sha256: "sha2".to_string(),
        });
        db.get_package_mut("com.example.app").unwrap().active = Some("1.0.0".to_string());
        db.save(&paths.packages_file, &paths.var_dir).unwrap();

        let cfg = Config { colors: "never".to_string(), ..Default::default() };
        let args = vec!["app".to_string(), "2.0.0".to_string()];
        execute_switch(&paths, &cfg, &args).unwrap();

        let reloaded = PackagesDatabase::load(&paths.packages_file).unwrap();
        assert_eq!(reloaded.find_by_id("com.example.app").unwrap().active, Some("2.0.0".to_string()));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
