use crate::config::paths::LaciePaths;
use crate::config::Config;
use crate::diag::DiagError;
use crate::filter::{matches_package_filter, parse_filter_args, PACKAGE_FILTER_KEYS};
use crate::locale;
use crate::state::packages::{PackageEntry, PackageVersionEntry, PackagesDatabase};
use crate::ui::color::{visible_len, ColorMode, Styler};
use crate::ui::progress::get_terminal_width;

fn truncate_ansi(s: &str, max_vis: usize) -> String {
    let mut out = String::new();
    let mut vis = 0;
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_esc = true;
            out.push(c);
        } else if in_esc {
            out.push(c);
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
        } else {
            if vis >= max_vis {
                break;
            }
            out.push(c);
            vis += 1;
        }
    }
    out.push_str("\x1b[0m");
    out
}

fn collect_package_content_lines(
    pkg: &PackageEntry,
    active_ver: Option<&PackageVersionEntry>,
    styler: &Styler,
) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(v) = active_ver {
        if let Some(ref auth) = v.author {
            lines.push(std::format!("{:<12} {}", locale::t("cmd_info.author_label"), auth));
        }
        let ver_line = std::format!(
            "{} {}",
            styler.bold_white(&v.version),
            styler.dim(&std::format!("(number {})", v.version_number))
        );
        lines.push(std::format!("{:<12} {}", locale::t("cmd_info.version_label"), ver_line));
        if let Some(ref sc) = v.source_code {
            lines.push(std::format!("{:<12} {}", locale::t("cmd_info.source_code_label"), styler.cyan(sc)));
        }
        if !v.links.is_empty() {
            lines.push(locale::t("cmd_info.links_label"));
            for link in &v.links {
                lines.push(std::format!("    - {}", styler.cyan(link)));
            }
        }
    }
    if pkg.command != pkg.name {
        lines.push(std::format!("{:<12} {}", locale::t("cmd_info.command_label"), styler.bold_white(&pkg.command)));
    }

    if !pkg.versions.is_empty() {
        lines.push(String::new());
        lines.push(styler.bold_white(&locale::t("cmd_info.installed_versions_label")));
        for (ver_str, ver_entry) in pkg.versions.iter().rev() {
            let status_suffix = if pkg.frozen && pkg.frozen_version.as_deref() == Some(ver_str) {
                std::format!(" {}", styler.cyan("(frozen)"))
            } else if pkg.active.as_deref() == Some(ver_str) {
                std::format!(" {}", styler.green("(active)"))
            } else {
                String::new()
            };
            lines.push(std::format!("  {}{}", styler.bold_white(ver_str), status_suffix));
            if let Some(ref dt) = ver_entry.installed_at {
                lines.push(std::format!("    {:<10} {}", locale::t("cmd_info.installed_label"), dt.format("%Y-%m-%d %H:%M")));
            }
            if let Some(ref dt) = ver_entry.released_at {
                lines.push(std::format!("    {:<10} {}", locale::t("cmd_info.released_label"), dt.format("%Y-%m-%d")));
            }
            let src_display = std::format!("{}:{}", ver_entry.source, ver_entry.storage);
            lines.push(std::format!("    {:<10} {}", locale::t("cmd_info.source_label"), src_display));
            lines.push(std::format!("    {:<10} {}", locale::t("cmd_info.sha256_label"), styler.dim(&ver_entry.sha256)));
        }
    }
    lines
}

fn print_single_package_info(pkg: &PackageEntry, id: &str, styler: &Styler) {
    let active_ver = pkg.active.as_ref()
        .or(pkg.frozen_version.as_ref())
        .and_then(|v| pkg.versions.get(v))
        .or_else(|| pkg.versions.values().next_back());
    println!("{} {}\n", styler.bold_white(&pkg.name), styler.dim(&std::format!("({})", id)));
    let lines = collect_package_content_lines(pkg, active_ver, styler);
    for line in lines {
        println!("  {}", line);
    }
}

fn print_framed_package_info(
    pkg: &PackageEntry,
    id: &str,
    styler: &Styler,
    term_width: usize,
) {
    if term_width < 45 {
        print_single_package_info(pkg, id, styler);
        return;
    }

    let active_ver = pkg.active.as_ref()
        .or(pkg.frozen_version.as_ref())
        .and_then(|v| pkg.versions.get(v))
        .or_else(|| pkg.versions.values().next_back());

    let lines = collect_package_content_lines(pkg, active_ver, styler);

    let title_clean_len = pkg.name.len() + id.len() + 7;
    let max_line_len = lines.iter().map(|l| visible_len(l)).max().unwrap_or(0);
    let needed = max_line_len.max(title_clean_len) + 4;
    let box_width = needed.min(term_width).max(45);
    let inner_width = box_width.saturating_sub(4);

    let title_colored = std::format!("── {} {} ", styler.bold_white(&pkg.name), styler.dim(&std::format!("({})", id)));
    let title_vis = visible_len(&title_colored);
    let top_fill = box_width.saturating_sub(title_vis + 2);
    println!("{}{}{}{}", styler.dim("┌"), title_colored, styler.dim(&"─".repeat(top_fill)), styler.dim("┐"));
    println!("{} {} {}", styler.dim("│"), " ".repeat(inner_width), styler.dim("│"));

    for line in &lines {
        let vis = visible_len(line);
        let (fitted_line, fitted_vis) = if vis > inner_width {
            let truncated = truncate_ansi(line, inner_width.saturating_sub(3));
            (std::format!("{}...", truncated), inner_width)
        } else {
            (line.clone(), vis)
        };
        let pad = inner_width.saturating_sub(fitted_vis);
        println!("{} {}{}{}", styler.dim("│"), fitted_line, " ".repeat(pad + 1), styler.dim("│"));
    }

    println!("{} {} {}", styler.dim("│"), " ".repeat(inner_width), styler.dim("│"));
    println!("{}{}{}", styler.dim("└"), styler.dim(&"─".repeat(box_width.saturating_sub(2))), styler.dim("┘"));
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

    let term_width = get_terminal_width();
    let is_multi = targets.len() > 1;

    for (i, (id, pkg)) in targets.iter().enumerate() {
        if is_multi {
            print_framed_package_info(pkg, id, &styler, term_width);
        } else {
            print_single_package_info(pkg, id, &styler);
        }
        if i + 1 < targets.len() {
            println!();
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

    #[test]
    fn test_truncate_ansi() {
        let colored = "\x1b[32mhello world\x1b[0m";
        let truncated = truncate_ansi(colored, 5);
        assert_eq!(visible_len(&truncated), 5);
    }

    #[test]
    fn test_framed_info_multiple() {
        let tmp = std::env::temp_dir().join("lacie_test_info_multi");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp.clone());
        paths.ensure_all_dirs().unwrap();
        let cfg = Config::default();

        let mut db = PackagesDatabase::default();
        let mut p1 = PackageEntry {
            name: "pkg-one".to_string(),
            command: "pkg1".to_string(),
            active: Some("1.0.0".to_string()),
            frozen: false,
            frozen_version: None,
            versions: std::collections::BTreeMap::new(),
        };
        p1.versions.insert("1.0.0".to_string(), PackageVersionEntry {
            version: "1.0.0".to_string(),
            version_number: 100,
            target: "linux-gnu-x86_64".to_string(),
            installed_at: None,
            released_at: None,
            source: "github".to_string(),
            storage: "owner/repo1".to_string(),
            sha256: "abc123".to_string(),
            author: Some("Author One".to_string()),
            source_code: Some("https://example.com/repo1".to_string()),
            links: vec![],
        });
        db.packages.insert("owner/repo1".to_string(), p1);

        let mut p2 = PackageEntry {
            name: "pkg-two".to_string(),
            command: "pkg2".to_string(),
            active: Some("2.0.0".to_string()),
            frozen: false,
            frozen_version: None,
            versions: std::collections::BTreeMap::new(),
        };
        p2.versions.insert("2.0.0".to_string(), PackageVersionEntry {
            version: "2.0.0".to_string(),
            version_number: 200,
            target: "linux-gnu-x86_64".to_string(),
            installed_at: None,
            released_at: None,
            source: "github".to_string(),
            storage: "owner/repo2".to_string(),
            sha256: "def456".to_string(),
            author: None,
            source_code: None,
            links: vec![],
        });
        db.packages.insert("owner/repo2".to_string(), p2);

        db.save(&paths.packages_file, &paths.var_dir).unwrap();

        assert!(execute_info(&paths, &cfg, &[]).is_ok());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
