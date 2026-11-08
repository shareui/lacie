use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::filter::{matches_package_filter, parse_filter_args, PACKAGE_FILTER_KEYS};
use crate::locale;
use crate::state::packages::{PackageEntry, PackageVersionEntry, PackagesDatabase};
use crate::ui::color::{ColorMode, Styler};

fn print_package_header(pkg: &PackageEntry, id: &str, active_ver: Option<&PackageVersionEntry>, styler: &Styler) {
    println!("{} {}\n", styler.bold_white(&pkg.name), styler.dim(&std::format!("({})", id)));
    if let Some(v) = active_ver {
        if let Some(ref auth) = v.author {
            println!("  {:<12} {}", locale::t("cmd_info.author_label"), auth);
        }
        let ver_line = std::format!("{} {}", styler.bold_white(&v.version), styler.dim(&std::format!("(number {})", v.version_number)));
        println!("  {:<12} {}", locale::t("cmd_info.version_label"), ver_line);
        if let Some(ref sc) = v.source_code {
            println!("  {:<12} {}", locale::t("cmd_info.source_code_label"), styler.cyan(sc));
        }
        if !v.links.is_empty() {
            println!("  {}", locale::t("cmd_info.links_label"));
            for link in &v.links {
                println!("      - {}", styler.cyan(link));
            }
        }
    }
    if pkg.command != pkg.name {
        println!("  {:<12} {}", locale::t("cmd_info.command_label"), styler.bold_white(&pkg.command));
    }
}

fn print_installed_versions(pkg: &PackageEntry, styler: &Styler) {
    println!("\n{}", styler.bold_white(&locale::t("cmd_info.installed_versions_label")));
    for (ver_str, ver_entry) in pkg.versions.iter().rev() {
        let status_suffix = if pkg.frozen && pkg.frozen_version.as_deref() == Some(ver_str) {
            std::format!(" {}", styler.cyan("(frozen)"))
        } else if pkg.active.as_deref() == Some(ver_str) {
            std::format!(" {}", styler.green("(active)"))
        } else {
            String::new()
        };
        println!("\n  {}{}", styler.bold_white(ver_str), status_suffix);
        if let Some(ref dt) = ver_entry.installed_at {
            println!("    {:<10} {}", locale::t("cmd_info.installed_label"), dt.format("%Y-%m-%d %H:%M"));
        }
        if let Some(ref dt) = ver_entry.released_at {
            println!("    {:<10} {}", locale::t("cmd_info.released_label"), dt.format("%Y-%m-%d"));
        }
        let src_display = std::format!("{}:{}", ver_entry.source, ver_entry.storage);
        println!("    {:<10} {}", locale::t("cmd_info.source_label"), src_display);
        println!("    {:<10} {}", locale::t("cmd_info.sha256_label"), styler.dim(&ver_entry.sha256));
    }
}

fn print_single_package_info(pkg: &PackageEntry, id: &str, styler: &Styler) {
    let active_ver = pkg.active.as_ref()
        .or(pkg.frozen_version.as_ref())
        .and_then(|v| pkg.versions.get(v))
        .or_else(|| pkg.versions.values().next_back());
    print_package_header(pkg, id, active_ver, styler);
    print_installed_versions(pkg, styler);
}

fn resolve_target_packages<'a>(
    db: &'a PackagesDatabase,
    query: Option<&str>,
) -> Result<Vec<(&'a String, &'a PackageEntry)>, DiagError> {
    if let Some(q) = query {
        let matches = db.find_by_name(q);
        if matches.len() > 1 {
            let ids: Vec<String> = matches.iter().map(|(id, _)| (*id).clone()).collect();
            return Err(DiagError::from_code("E012", &[("name", q), ("ids", &ids.join(", "))]));
        }
        if let Some((id, pkg)) = matches.first() {
            return Ok(vec![(*id, *pkg)]);
        }
        if let Some(pkg) = db.find_by_id(q) {
            let id = db.packages.keys().find(|k| k.as_str() == q).unwrap();
            return Ok(vec![(id, pkg)]);
        }
        return Err(DiagError::from_code("E009", &[("name", q)]));
    }
    Ok(db.packages.iter().collect())
}

pub fn execute_info(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let is_json = args.iter().any(|a| a == "--json");
    let rules = parse_filter_args(args, PACKAGE_FILTER_KEYS, "info")?;
    let db = PackagesDatabase::load(&paths.packages_file)?;
    let styler = Styler::new(cfg.colors.parse().unwrap_or(ColorMode::Terminal));

    if db.packages.is_empty() {
        if is_json {
            println!("[]");
        } else {
            println!("{}", locale::t("cmd_list.no_packages"));
        }
        return Ok(());
    }

    let non_flags: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if non_flags.len() > 1 {
        return Err(DiagError::from_code("E017", &[("expected", "at most one package name")]));
    }
    let query = non_flags.first().map(|s| s.as_str());
    let mut targets = resolve_target_packages(&db, query)?;
    targets.retain(|(id, pkg)| matches_package_filter(pkg, id, &rules));

    if targets.is_empty() {
        if is_json {
            println!("[]");
        } else {
            println!("{}", locale::t("cmd_list.no_matches"));
        }
        return Ok(());
    }

    if is_json {
        let json_arr: Vec<serde_json::Value> = targets.iter().map(|(id, pkg)| {
            serde_json::json!({
                "id": id,
                "name": pkg.name,
                "command": pkg.command,
                "active": pkg.active,
                "frozen": pkg.frozen,
                "versions": pkg.versions,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&json_arr).unwrap_or_default());
        return Ok(());
    }

    for (i, (id, pkg)) in targets.iter().enumerate() {
        print_single_package_info(pkg, id, &styler);
        if i + 1 < targets.len() {
            println!("\n");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_empty() {
        let tmp = std::env::temp_dir().join("lacie_test_info_cmd");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let cfg = Config::default();

        assert!(execute_info(&paths, &cfg, &[]).is_ok());
    }
}
