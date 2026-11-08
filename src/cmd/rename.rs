use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::locale;
use crate::state::packages::PackagesDatabase;

fn validate_command_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    if name.starts_with('-') || name.starts_with('.') {
        return false;
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return false;
    }
    let upper = name.to_ascii_uppercase();
    if upper == "LACIE" {
        return false;
    }
    let win_reserved = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    !win_reserved.contains(&upper.as_str())
}

fn rename_active_binary(paths: &LaciePaths, old_cmd: &str, new_cmd: &str) -> Result<(), DiagError> {
    #[cfg(unix)]
    {
        let old_link = paths.active_dir.join(old_cmd);
        let new_link = paths.active_dir.join(new_cmd);
        if old_link.exists() || old_link.is_symlink() {
            let target = std::fs::read_link(&old_link).map_err(|e| DiagError::new("E015", e.to_string()))?;
            let _ = std::fs::remove_file(&old_link);
            if new_link.exists() || new_link.is_symlink() {
                let _ = std::fs::remove_file(&new_link);
            }
            std::os::unix::fs::symlink(target, new_link).map_err(|e| DiagError::new("E015", e.to_string()))?;
        }
    }
    #[cfg(windows)]
    {
        let old_name = std::format!("{}.exe", old_cmd.trim_end_matches(".exe"));
        let new_name = std::format!("{}.exe", new_cmd.trim_end_matches(".exe"));
        let old_file = paths.active_dir.join(old_name);
        let new_file = paths.active_dir.join(new_name);
        if old_file.exists() {
            std::fs::rename(old_file, new_file).map_err(|e| DiagError::new("E015", e.to_string()))?;
        }
    }
    let _ = (paths, old_cmd, new_cmd);
    Ok(())
}

fn resolve_single_pkg(query: &str, db: &PackagesDatabase) -> Result<String, DiagError> {
    if query == "ALL" {
        return Err(DiagError::from_code("E017", &[("expected", "single package name")]));
    }
    let matches = db.find_by_name(query);
    if matches.len() > 1 {
        let ids: Vec<String> = matches.iter().map(|(id, _)| (*id).clone()).collect();
        return Err(DiagError::from_code("E012", &[("name", query), ("ids", &ids.join(", "))]));
    }
    if let Some((id, _)) = matches.first() {
        return Ok((*id).clone());
    }
    if db.find_by_id(query).is_some() {
        return Ok(query.to_string());
    }
    Err(DiagError::from_code("E009", &[("name", query)]))
}

pub fn execute_rename(paths: &LaciePaths, _cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    if args.len() != 2 {
        return Err(DiagError::from_code("E017", &[("expected", "package and command name")]));
    }
    let mut db = PackagesDatabase::load(&paths.packages_file)?;
    let id = resolve_single_pkg(&args[0], &db)?;
    let is_default = args[1] == "DEFAULT";
    let (pkg_name, old_cmd, is_active) = {
        let pkg = db.find_by_id(&id).unwrap();
        (pkg.name.clone(), pkg.command.clone(), pkg.active.is_some() && !pkg.frozen)
    };

    let new_cmd = if is_default { &pkg_name } else { &args[1] };
    if !validate_command_name(new_cmd) {
        return Err(DiagError::from_code("E004", &[("key", "command"), ("value", new_cmd), ("allowed", "valid command name")]));
    }

    for (other_id, other) in &db.packages {
        if other_id != &id && other.command.eq_ignore_ascii_case(new_cmd) {
            return Err(DiagError::from_code("E027", &[("name", new_cmd), ("package", &other.name)]));
        }
    }

    if is_active && old_cmd != *new_cmd {
        rename_active_binary(paths, &old_cmd, new_cmd)?;
    }

    if let Some(pkg) = db.get_package_mut(&id) {
        pkg.command = new_cmd.to_string();
    }
    db.save(&paths.packages_file, &paths.var_dir)?;

    let msg = if is_default {
        locale::format("cmd_rename.renamed_default", &[("package", &pkg_name), ("old", &old_cmd), ("new", new_cmd)])
    } else if !is_active {
        locale::format("cmd_rename.renamed_inactive", &[("package", &pkg_name), ("old", &old_cmd), ("new", new_cmd)])
    } else {
        locale::format("cmd_rename.renamed", &[("package", &pkg_name), ("old", &old_cmd), ("new", new_cmd)])
    };
    println!("{}", msg);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::packages::PackageVersionEntry;

    #[test]
    fn test_rename_command_validation() {
        assert!(validate_command_name("my-cmd"));
        assert!(validate_command_name("tool_1.0"));
        assert!(!validate_command_name("lacie"));
        assert!(!validate_command_name("LACIE"));
        assert!(!validate_command_name("-cmd"));
        assert!(!validate_command_name(".cmd"));
        assert!(!validate_command_name("CON"));
        assert!(!validate_command_name("com1"));
        assert!(!validate_command_name("invalid char"));
    }

    #[test]
    fn test_rename_lifecycle() {
        let tmp = std::env::temp_dir().join("lacie_test_rename");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let cfg = Config::default();

        let mut db = PackagesDatabase::default();
        let bin_dir = paths.bin_dir.join("com.example.tool").join("1.0.0");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let bin_file = bin_dir.join("tool-sha1");
        std::fs::write(&bin_file, "binary").unwrap();
        let active_symlink = paths.active_dir.join("tool");
        crate::platform::fs::activate_binary(&bin_file, &active_symlink).unwrap();

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
            sha256: "sha1".to_string(),
        });
        if let Some(pkg) = db.get_package_mut("com.example.tool") {
            pkg.active = Some("1.0.0".to_string());
        }
        db.save(&paths.packages_file, &paths.var_dir).unwrap();

        execute_rename(&paths, &cfg, &["tool".to_string(), "mytool".to_string()]).unwrap();
        let db2 = PackagesDatabase::load(&paths.packages_file).unwrap();
        assert_eq!(db2.find_by_id("com.example.tool").unwrap().command, "mytool");

        execute_rename(&paths, &cfg, &["tool".to_string(), "DEFAULT".to_string()]).unwrap();
        let db3 = PackagesDatabase::load(&paths.packages_file).unwrap();
        assert_eq!(db3.find_by_id("com.example.tool").unwrap().command, "tool");
    }
}
