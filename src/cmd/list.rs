use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::filter::{matches_package_filter, parse_filter_args, PACKAGE_FILTER_KEYS};
use crate::locale;
use crate::state::packages::PackagesDatabase;
use crate::ui::color::{ColorMode, Styler};

struct ListRow<'a> {
    pub name: &'a str,
    pub id: &'a str,
    pub ver_raw: String,
    pub is_frozen: bool,
}

pub fn execute_list(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let is_json = args.iter().any(|a| a == "--json");
    let rules = parse_filter_args(args, PACKAGE_FILTER_KEYS, "list")?;
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

    let mut rows = Vec::new();
    for (id, pkg) in &db.packages {
        if !matches_package_filter(pkg, id, &rules) {
            continue;
        }
        let is_frozen = pkg.frozen;
        let ver_raw = if pkg.frozen {
            pkg.frozen_version.clone().unwrap_or_else(|| "-".to_string())
        } else if let Some(ref a) = pkg.active {
            a.clone()
        } else {
            "-".to_string()
        };
        rows.push(ListRow {
            name: &pkg.name,
            id,
            ver_raw,
            is_frozen,
        });
    }

    if rows.is_empty() {
        if is_json {
            println!("[]");
        } else {
            println!("{}", locale::t("cmd_list.no_matches"));
        }
        return Ok(());
    }

    if is_json {
        let json_arr: Vec<serde_json::Value> = rows.iter().map(|r| {
            let ver_display = if r.is_frozen {
                std::format!("{} (frozen)", r.ver_raw)
            } else {
                r.ver_raw.clone()
            };
            serde_json::json!({
                "name": r.name,
                "id": r.id,
                "version": ver_display,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&json_arr).unwrap_or_default());
        return Ok(());
    }

    rows.sort_by_key(|r| r.name);
    let w_name = rows.iter().map(|r| r.name.len()).max().unwrap_or(4);
    let w_ver = rows.iter().map(|r| {
        if r.is_frozen { r.ver_raw.len() + 9 } else { r.ver_raw.len() }
    }).max().unwrap_or(5);

    println!();
    for r in rows {
        let name_colored = styler.bold_white(r.name);
        let pad_name = " ".repeat(w_name.saturating_sub(r.name.len()));

        let (ver_colored, ver_len) = if r.ver_raw == "-" {
            (styler.dim("-"), 1)
        } else if r.is_frozen {
            (std::format!("{} {}", r.ver_raw, styler.cyan("(frozen)")), r.ver_raw.len() + 9)
        } else {
            (styler.green(&r.ver_raw), r.ver_raw.len())
        };
        let pad_ver = " ".repeat(w_ver.saturating_sub(ver_len));
        let id_colored = styler.dim(r.id);

        println!("{}{}  {}{}  {}", name_colored, pad_name, ver_colored, pad_ver, id_colored);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::packages::PackageVersionEntry;

    #[test]
    fn test_list_empty_and_populated() {
        let tmp = std::env::temp_dir().join("lacie_test_list_cmd");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let cfg = Config::default();

        assert!(execute_list(&paths, &cfg, &[]).is_ok());

        let mut db = PackagesDatabase::default();
        db.insert_version("com.example.app", "app", Some("1.0.0"), PackageVersionEntry {
            version: "1.0.0".to_string(),
            version_number: 1,
            author: Some("shareui".to_string()),
            links: vec![],
            source_code: None,
            installed_at: None,
            released_at: None,
            source: "github".to_string(),
            storage: "repo".to_string(),
            target: "x86_64".to_string(),
            sha256: "hash".to_string(),
        });
        db.save(&paths.packages_file, &paths.var_dir).unwrap();

        assert!(execute_list(&paths, &cfg, &[]).is_ok());
        assert!(execute_list(&paths, &cfg, &["--json".to_string()]).is_ok());
        assert!(execute_list(&paths, &cfg, &["--filter=name:app".to_string()]).is_ok());
        assert!(execute_list(&paths, &cfg, &["--filter=name:nomatch".to_string()]).is_ok());
    }
}
