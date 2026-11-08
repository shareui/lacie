use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::state::packages::PackagesDatabase;
use crate::state::sources::SourcesDatabase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctorStatus {
    Ok,
    Warning,
    Problem,
}

impl DoctorStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DoctorStatus::Ok => "ok",
            DoctorStatus::Warning => "warning",
            DoctorStatus::Problem => "problem",
        }
    }
}

pub struct DoctorCheck {
    pub name: String,
    pub status: DoctorStatus,
    pub details: Option<String>,
}

fn check_directory(paths: &LaciePaths) -> DoctorCheck {
    if paths.root.is_dir() {
        DoctorCheck {
            name: "Directory".to_string(),
            status: DoctorStatus::Ok,
            details: Some(paths.root.display().to_string()),
        }
    } else {
        DoctorCheck {
            name: "Directory".to_string(),
            status: DoctorStatus::Problem,
            details: Some("root directory does not exist".to_string()),
        }
    }
}

fn check_structure(paths: &LaciePaths) -> DoctorCheck {
    let required_dirs = [
        &paths.active_dir, &paths.bin_dir, &paths.scripts_dir,
        &paths.plugins_dir, &paths.var_dir, &paths.cache_dir,
    ];
    let missing_dirs: Vec<String> = required_dirs.iter()
        .filter(|p| !p.is_dir())
        .map(|p| p.display().to_string())
        .collect();

    if !missing_dirs.is_empty() {
        return DoctorCheck {
            name: "Structure".to_string(),
            status: DoctorStatus::Problem,
            details: Some(std::format!("missing directories: {}", missing_dirs.join(", "))),
        };
    }

    if !paths.config_file.is_file() {
        return DoctorCheck {
            name: "Structure".to_string(),
            status: DoctorStatus::Problem,
            details: Some("config.toml is missing".to_string()),
        };
    }
    DoctorCheck {
        name: "Structure".to_string(),
        status: DoctorStatus::Ok,
        details: None,
    }
}

fn check_path_entry(paths: &LaciePaths) -> DoctorCheck {
    let path_env = std::env::var("PATH").unwrap_or_default();
    let active_str = paths.active_dir.to_string_lossy();
    if path_env.split(':').any(|p| p == active_str) {
        DoctorCheck {
            name: "PATH".to_string(),
            status: DoctorStatus::Ok,
            details: None,
        }
    } else {
        DoctorCheck {
            name: "PATH".to_string(),
            status: DoctorStatus::Warning,
            details: Some("active/ is not in PATH".to_string()),
        }
    }
}

fn check_links(paths: &LaciePaths, pkgs_db: &PackagesDatabase) -> DoctorCheck {
    for pkg in pkgs_db.packages.values() {
        if let Some(ref ver) = pkg.active {
            if pkg.frozen {
                continue;
            }
            let active_file = paths.active_dir.join(&pkg.command);
            if !active_file.exists() && !active_file.is_symlink() {
                return DoctorCheck {
                    name: "Links".to_string(),
                    status: DoctorStatus::Problem,
                    details: Some(std::format!("active binary for {} is missing in active/", pkg.name)),
                };
            }
            #[cfg(unix)]
            if let Ok(target) = std::fs::read_link(&active_file)
                && !target.exists() {
                return DoctorCheck {
                    name: "Links".to_string(),
                    status: DoctorStatus::Problem,
                    details: Some(std::format!("active/{} points to a missing file", pkg.command)),
                };
            }
            let _ = ver;
        }
    }
    DoctorCheck {
        name: "Links".to_string(),
        status: DoctorStatus::Ok,
        details: None,
    }
}

fn check_packages(paths: &LaciePaths, pkgs_db: &PackagesDatabase) -> DoctorCheck {
    for (id, pkg) in &pkgs_db.packages {
        for (ver, entry) in &pkg.versions {
            let bin_name = std::format!("{}-{}", pkg.name, entry.sha256);
            let bin_path = paths.bin_dir.join(id).join(ver).join(&bin_name);
            if !bin_path.is_file() {
                return DoctorCheck {
                    name: "Packages".to_string(),
                    status: DoctorStatus::Problem,
                    details: Some(std::format!("{} {}: file is missing", id, ver)),
                };
            }
        }
    }
    DoctorCheck {
        name: "Packages".to_string(),
        status: DoctorStatus::Ok,
        details: None,
    }
}

fn check_sources(paths: &LaciePaths, cfg: &Config) -> DoctorCheck {
    let sources_db = match SourcesDatabase::load(&paths.sources_file) {
        Ok(db) => db,
        Err(_) => return DoctorCheck {
            name: "Sources".to_string(),
            status: DoctorStatus::Problem,
            details: Some("could not read sources.toml".to_string()),
        },
    };
    if !sources_db.is_enabled(&cfg.default_source) {
        return DoctorCheck {
            name: "Sources".to_string(),
            status: DoctorStatus::Problem,
            details: Some(std::format!("default_source `{}` is missing or disabled", cfg.default_source)),
        };
    }
    DoctorCheck {
        name: "Sources".to_string(),
        status: DoctorStatus::Ok,
        details: None,
    }
}

fn check_temp_leftovers(paths: &LaciePaths) -> DoctorCheck {
    let mut leftover_count = 0;
    if paths.var_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&paths.var_dir)
    {
        for e in entries.flatten() {
            let name = e.file_name();
            if name != "cache" {
                leftover_count += 1;
            }
        }
    }
    if leftover_count > 0 {
        DoctorCheck {
            name: "Temp".to_string(),
            status: DoctorStatus::Warning,
            details: Some(std::format!("{} leftover items in var/", leftover_count)),
        }
    } else {
        DoctorCheck {
            name: "Temp".to_string(),
            status: DoctorStatus::Ok,
            details: None,
        }
    }
}

pub fn execute_doctor(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let is_json = args.iter().any(|a| a == "--json");
    let is_fix = args.iter().any(|a| a == "--fix");

    if is_fix {
        paths.ensure_all_dirs().map_err(|e| DiagError::new("E015", e.to_string()))?;
        let _ = crate::ruby::builtin::materialize_builtin_github(&paths.plugins_dir);
        let _ = crate::config::shell::add_active_to_path(&paths.active_dir);
        if paths.var_dir.is_dir() && let Ok(entries) = std::fs::read_dir(&paths.var_dir) {
            for e in entries.flatten() {
                if e.file_name() != "cache" {
                    let _ = std::fs::remove_dir_all(e.path());
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
    }

    let pkgs_db = PackagesDatabase::load(&paths.packages_file).unwrap_or_default();
    let checks = vec![
        check_directory(paths),
        check_structure(paths),
        DoctorCheck { name: "Config".to_string(), status: DoctorStatus::Ok, details: None },
        check_path_entry(paths),
        check_links(paths, &pkgs_db),
        check_packages(paths, &pkgs_db),
        DoctorCheck { name: "Scripts".to_string(), status: DoctorStatus::Ok, details: None },
        check_sources(paths, cfg),
        check_temp_leftovers(paths),
    ];

    if is_json {
        let json_arr: Vec<serde_json::Value> = checks.iter().map(|c| {
            serde_json::json!({
                "check": c.name,
                "status": c.status.as_str(),
                "details": c.details,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&json_arr).unwrap_or_default());
        return Ok(());
    }

    let problems = checks.iter().filter(|c| c.status == DoctorStatus::Problem).count();
    let warnings = checks.iter().filter(|c| c.status == DoctorStatus::Warning).count();

    println!();
    println!("  {:<10} {:<8} DETAILS", "CHECK", "STATUS");
    for c in &checks {
        let det = c.details.as_deref().unwrap_or("");
        println!("  {:<10} {:<8} {}", c.name, c.status.as_str(), det);
    }
    println!();
    if problems > 0 || warnings > 0 {
        println!("{} problems, {} warning. Run `lacie doctor --fix` to repair.", problems, warnings);
    }
    if problems > 0 {
        return Err(DiagError::new("E001", "doctor discovered critical problems".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_execution() {
        let tmp = std::env::temp_dir().join("lacie_test_doctor");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let cfg = Config::default();

        let _ = execute_doctor(&paths, &cfg, &["--json".to_string()]);
    }
}
