use crate::diag::DiagError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterVal {
    Exact(String),
    Range(Option<String>, Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterRule {
    pub key: String,
    pub is_not: bool,
    pub values: Vec<FilterVal>,
}

fn parse_single_val(raw: &str) -> FilterVal {
    if let Some((min, max)) = raw.split_once("..") {
        let min_opt = if min.is_empty() { None } else { Some(min.to_string()) };
        let max_opt = if max.is_empty() { None } else { Some(max.to_string()) };
        FilterVal::Range(min_opt, max_opt)
    } else {
        FilterVal::Exact(raw.to_string())
    }
}

pub fn parse_filter(raw: &str, allowed_keys: &[&str], cmd: &str) -> Result<FilterRule, DiagError> {
    let clean = raw.strip_prefix("--filter=").unwrap_or(raw);
    let (key_part, val_part) = clean.split_once(':')
        .ok_or_else(|| DiagError::from_code("E021", &[("filter", raw), ("command", cmd), ("keys", &allowed_keys.join(", "))]))?;

    let (is_not, base_key) = if let Some(stripped) = key_part.strip_prefix("not-") {
        (true, stripped)
    } else {
        (false, key_part)
    };

    if !allowed_keys.contains(&base_key) {
        return Err(DiagError::from_code("E021", &[("filter", raw), ("command", cmd), ("keys", &allowed_keys.join(", "))]));
    }

    let values: Vec<FilterVal> = val_part.split(',').map(parse_single_val).collect();
    if values.is_empty() {
        return Err(DiagError::from_code("E004", &[("key", base_key), ("value", val_part), ("allowed", "non-empty value")]));
    }

    Ok(FilterRule {
        key: base_key.to_string(),
        is_not,
        values,
    })
}

pub fn parse_filter_args(args: &[String], allowed_keys: &[&str], cmd: &str) -> Result<Vec<FilterRule>, DiagError> {
    let mut rules = Vec::new();
    for arg in args {
        if arg.starts_with("--filter=") || (arg.contains(':') && !arg.starts_with('-')) {
            rules.push(parse_filter(arg, allowed_keys, cmd)?);
        }
    }
    Ok(rules)
}

pub fn check_text_match(actual: &str, rule: &FilterRule) -> bool {
    let matches = rule.values.iter().any(|v| match v {
        FilterVal::Exact(exp) => actual.to_lowercase().contains(&exp.to_lowercase()),
        _ => false,
    });
    if rule.is_not { !matches } else { matches }
}

pub fn check_exact_match(actual: &str, rule: &FilterRule) -> bool {
    let matches = rule.values.iter().any(|v| match v {
        FilterVal::Exact(exp) => actual.eq_ignore_ascii_case(exp),
        _ => false,
    });
    if rule.is_not { !matches } else { matches }
}

pub fn check_bool_match(actual: bool, rule: &FilterRule) -> bool {
    let matches = rule.values.iter().any(|v| match v {
        FilterVal::Exact(exp) => {
            let lower = exp.to_lowercase();
            let exp_b = lower == "yes" || lower == "true";
            actual == exp_b
        }
        _ => false,
    });
    if rule.is_not { !matches } else { matches }
}

pub fn check_number_match(actual: u64, rule: &FilterRule) -> bool {
    let matches = rule.values.iter().any(|v| match v {
        FilterVal::Exact(exp) => exp.parse::<u64>().map(|n| n == actual).unwrap_or(false),
        FilterVal::Range(min, max) => {
            let min_ok = min.as_deref().and_then(|m| m.parse::<u64>().ok()).map(|m| actual >= m).unwrap_or(true);
            let max_ok = max.as_deref().and_then(|m| m.parse::<u64>().ok()).map(|m| actual <= m).unwrap_or(true);
            min_ok && max_ok
        }
    });
    if rule.is_not { !matches } else { matches }
}

pub fn check_date_match(actual: Option<&DateTime<Utc>>, rule: &FilterRule) -> bool {
    let dt = match actual {
        Some(d) => d,
        None => return rule.is_not,
    };
    let actual_str = dt.format("%Y-%m-%d").to_string();
    let matches = rule.values.iter().any(|v| match v {
        FilterVal::Exact(exp) => actual_str.starts_with(exp),
        FilterVal::Range(min, max) => {
            let min_ok = min.as_deref().map(|m| actual_str.as_str() >= m).unwrap_or(true);
            let max_ok = max.as_deref().map(|m| actual_str.as_str() <= m).unwrap_or(true);
            min_ok && max_ok
        }
    });
    if rule.is_not { !matches } else { matches }
}

pub fn check_hash_prefix_match(actual: &str, rule: &FilterRule) -> bool {
    let matches = rule.values.iter().any(|v| match v {
        FilterVal::Exact(exp) => actual.to_lowercase().starts_with(&exp.to_lowercase()),
        _ => false,
    });
    if rule.is_not { !matches } else { matches }
}

pub static PACKAGE_FILTER_KEYS: &[&str] = &[
    "name", "command", "id", "author", "link", "source", "storage",
    "version", "number", "installed", "released", "hash", "versions",
    "active", "frozen", "unused",
];

pub fn matches_package_filter(
    pkg: &crate::state::packages::PackageEntry,
    id: &str,
    rules: &[FilterRule],
) -> bool {
    rules.iter().all(|r| match r.key.as_str() {
        "name" => check_text_match(&pkg.name, r),
        "command" => check_text_match(&pkg.command, r),
        "id" => check_text_match(id, r),
        "active" => check_bool_match(pkg.active.is_some() && !pkg.frozen, r),
        "frozen" => check_bool_match(pkg.frozen, r),
        "versions" => check_number_match(pkg.versions.len() as u64, r),
        "unused" => {
            let has_unused = pkg.versions.keys().any(|v| Some(v) != pkg.active.as_ref());
            check_bool_match(has_unused, r)
        }
        "version" => {
            let m = pkg.versions.keys().any(|v| check_exact_match(v, r));
            if r.is_not { !m } else { m }
        }
        "number" => {
            let m = pkg.versions.values().any(|v| check_number_match(v.version_number, r));
            if r.is_not { !m } else { m }
        }
        "author" => {
            let m = pkg.versions.values().any(|v| v.author.as_ref().map(|a| check_text_match(a, r)).unwrap_or(false));
            if r.is_not { !m } else { m }
        }
        "link" => {
            let m = pkg.versions.values().any(|v| v.links.iter().any(|l| check_text_match(l, r)));
            if r.is_not { !m } else { m }
        }
        "source" => {
            let m = pkg.versions.values().any(|v| check_exact_match(&v.source, r));
            if r.is_not { !m } else { m }
        }
        "storage" => {
            let m = pkg.versions.values().any(|v| check_text_match(&v.storage, r));
            if r.is_not { !m } else { m }
        }
        "installed" => {
            let m = pkg.versions.values().any(|v| check_date_match(v.installed_at.as_ref(), r));
            if r.is_not { !m } else { m }
        }
        "released" => {
            let m = pkg.versions.values().any(|v| check_date_match(v.released_at.as_ref(), r));
            if r.is_not { !m } else { m }
        }
        "hash" => {
            let m = pkg.versions.values().any(|v| check_hash_prefix_match(&v.sha256, r));
            if r.is_not { !m } else { m }
        }
        _ => true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_parsing_and_text_matching() {
        let rule = parse_filter("--filter=name:lacie,tool", &["name"], "list").unwrap();
        assert_eq!(rule.key, "name");
        assert!(!rule.is_not);
        assert!(check_text_match("io.lacie.app", &rule));
        assert!(check_text_match("mytool", &rule));
        assert!(!check_text_match("other", &rule));
    }

    #[test]
    fn test_filter_negation_and_number_range() {
        let rule = parse_filter("not-number:10..20", &["number"], "list").unwrap();
        assert!(rule.is_not);
        assert!(!check_number_match(15, &rule));
        assert!(check_number_match(25, &rule));
    }

    #[test]
    fn test_invalid_filter_key() {
        let err = parse_filter("unknown:val", &["name"], "list").unwrap_err();
        assert_eq!(err.code, "E021");
    }
}
