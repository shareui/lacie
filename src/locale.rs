use std::sync::{LazyLock, RwLock};
use toml::Table;

pub const EN_TOML: &str = include_str!("../locales/en.toml");
pub const RU_TOML: &str = include_str!("../locales/ru.toml");

static CURRENT_LANGUAGE: RwLock<Option<String>> = RwLock::new(None);

static EN_TABLE: LazyLock<Option<Table>> = LazyLock::new(|| {
    EN_TOML.parse::<Table>().ok()
});

static RU_TABLE: LazyLock<Option<Table>> = LazyLock::new(|| {
    RU_TOML.parse::<Table>().ok()
});

pub fn set_language(lang: &str) {
    if let Ok(mut current) = CURRENT_LANGUAGE.write() {
        *current = Some(lang.to_string());
    }
}

pub fn detect_system_language() -> &'static str {
    let check_var = |var: &str| -> Option<String> {
        std::env::var(var).ok().filter(|v| !v.is_empty())
    };
    let locale_str = check_var("LC_ALL")
        .or_else(|| check_var("LC_MESSAGES"))
        .or_else(|| check_var("LANG"))
        .unwrap_or_default()
        .to_lowercase();
    if locale_str.starts_with("ru") { "ru" } else { "en" }
}

pub fn resolve_language(configured: &str) -> &'static str {
    match configured {
        "ru" => "ru",
        "en" => "en",
        _ => detect_system_language(),
    }
}

pub fn current_language() -> &'static str {
    let guard = CURRENT_LANGUAGE.read().ok();
    let configured = guard.as_ref().and_then(|g| g.as_deref()).unwrap_or("auto");
    resolve_language(configured)
}

fn lookup_table<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current_val = table;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            return current_val.get(*part).and_then(|v| v.as_str());
        }
        current_val = current_val.get(*part).and_then(|v| v.as_table())?;
    }
    None
}

pub fn t(key: &str) -> String {
    let lang = current_language();
    if lang == "ru"
        && let Some(ref ru) = *RU_TABLE
        && let Some(val) = lookup_table(ru, key)
    {
        return val.to_string();
    }
    if let Some(ref en) = *EN_TABLE
        && let Some(val) = lookup_table(en, key)
    {
        return val.to_string();
    }
    key.to_string()
}

pub fn format(key: &str, args: &[(&str, &str)]) -> String {
    let mut template = t(key);
    for (name, value) in args {
        let needle = std::format!("{{{}}}", name);
        template = template.replace(&needle, value);
    }
    template
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_fallback() {
        set_language("ru");
        assert_eq!(t("general.yes"), "y");
        assert_eq!(t("cmd_init.q1_title"), "[1/5] Язык");
        assert_eq!(t("test_fallback.only_in_en"), "English fallback");
        assert_eq!(t("non_existing_key"), "non_existing_key");
    }

    #[test]
    fn test_format_parameters() {
        set_language("en");
        let result = format("cmd_init.lacie_dir", &[("dir", "/test/lacie")]);
        assert_eq!(result, "lacie directory: /test/lacie");
    }
}
