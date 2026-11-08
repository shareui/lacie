use crate::config::paths::LaciePaths;
use crate::config::{save_config, Config};
use crate::diag::DiagError;
use crate::locale;
use crate::state::sources::SourcesDatabase;
use crate::ui::color::{ColorMode, Styler};
use crate::ui::prompt::prompt_yes_no;

struct ConfigField {
    pub key: &'static str,
    pub value: String,
    pub default: &'static str,
}

fn get_config_fields(cfg: &Config) -> Vec<ConfigField> {
    vec![
        ConfigField { key: "language", value: cfg.language.clone(), default: "auto" },
        ConfigField { key: "colors", value: cfg.colors.clone(), default: "terminal" },
        ConfigField { key: "verify_sha256", value: cfg.verify_sha256.to_string(), default: "true" },
        ConfigField { key: "auto_switch", value: cfg.auto_switch.to_string(), default: "true" },
        ConfigField { key: "confirm", value: cfg.confirm.to_string(), default: "true" },
        ConfigField { key: "always_show", value: cfg.always_show.to_string(), default: "false" },
        ConfigField { key: "cache_ttl", value: cfg.cache_ttl.to_string(), default: "10" },
        ConfigField { key: "default_source", value: cfg.default_source.clone(), default: "github" },
        ConfigField { key: "windows_priority", value: cfg.windows_priority.clone(), default: "msvc" },
    ]
}

fn config_list(cfg: &Config, styler: &Styler) -> Result<(), DiagError> {
    let fields = get_config_fields(cfg);
    let pad_key_hdr = " ".repeat(18usize.saturating_sub(3));
    let pad_val_hdr = " ".repeat(10usize.saturating_sub(5));
    println!("\n  {}{}{}{}DEFAULT", styler.bold_white("KEY"), pad_key_hdr, styler.bold_white("VALUE"), pad_val_hdr);
    let mut any_diff = false;
    for f in &fields {
        let is_diff = f.value != f.default;
        if is_diff { any_diff = true; }
        let marker = if is_diff { styler.bold_cyan("*") } else { " ".to_string() };
        let val_str = if is_diff { styler.bold_green(&f.value) } else { f.value.clone() };
        let def_str = styler.dim(f.default);
        let pad_key = " ".repeat(18usize.saturating_sub(f.key.len()));
        let pad_val = " ".repeat(10usize.saturating_sub(f.value.len()));
        println!("{} {}{}{}{}{}", marker, f.key, pad_key, val_str, pad_val, def_str);
    }
    if any_diff {
        println!("\n{} {}", styler.bold_cyan("*"), styler.dim("differs from default"));
    }
    Ok(())
}

fn config_get(cfg: &Config, key: &str) -> Result<(), DiagError> {
    let fields = get_config_fields(cfg);
    if let Some(f) = fields.iter().find(|f| f.key == key) {
        println!("{}", f.value);
        Ok(())
    } else {
        Err(DiagError::from_code("E003", &[("key", key)]))
    }
}

fn apply_config_value(
    cfg: &mut Config,
    key: &str,
    val: &str,
    paths: &LaciePaths,
) -> Result<(), DiagError> {
    let sources_db = SourcesDatabase::load(&paths.sources_file)?;
    match key {
        "language" => {
            let v = if val == "DEFAULT" { "auto" } else { val };
            if !matches!(v, "auto" | "en" | "ru") {
                return Err(DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "auto, en, ru")]));
            }
            cfg.language = v.to_string();
        }
        "colors" => {
            let v = if val == "DEFAULT" { "terminal" } else { val };
            if !matches!(v, "terminal" | "always" | "never") {
                return Err(DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "terminal, always, never")]));
            }
            cfg.colors = v.to_string();
        }
        "verify_sha256" => {
            cfg.verify_sha256 = if val == "DEFAULT" { true } else { val.parse::<bool>().map_err(|_| DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "true, false")]))? };
        }
        "auto_switch" => {
            cfg.auto_switch = if val == "DEFAULT" { true } else { val.parse::<bool>().map_err(|_| DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "true, false")]))? };
        }
        "confirm" => {
            cfg.confirm = if val == "DEFAULT" { true } else { val.parse::<bool>().map_err(|_| DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "true, false")]))? };
        }
        "always_show" => {
            cfg.always_show = if val == "DEFAULT" { false } else { val.parse::<bool>().map_err(|_| DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "true, false")]))? };
        }
        "cache_ttl" => {
            cfg.cache_ttl = if val == "DEFAULT" { 10 } else { val.parse::<u64>().map_err(|_| DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "positive integer")]))? };
        }
        "default_source" => {
            let v = if val == "DEFAULT" { "github" } else { val };
            if !sources_db.sources.contains_key(v) {
                return Err(DiagError::from_code("E008", &[("source", v)]));
            }
            cfg.default_source = v.to_string();
        }
        "windows_priority" => {
            let v = if val == "DEFAULT" { "msvc" } else { val };
            if !matches!(v, "msvc" | "mingw") {
                return Err(DiagError::from_code("E004", &[("key", key), ("value", val), ("allowed", "msvc, mingw")]));
            }
            cfg.windows_priority = v.to_string();
        }
        _ => return Err(DiagError::from_code("E003", &[("key", key)])),
    }
    Ok(())
}

fn config_set(
    paths: &LaciePaths,
    cfg: &Config,
    key: &str,
    val: &str,
    styler: &Styler,
) -> Result<(), DiagError> {
    let fields = get_config_fields(cfg);
    let old_val = fields.iter().find(|f| f.key == key).map(|f| f.value.as_str()).unwrap_or("");
    let is_default = val == "DEFAULT";
    let mut updated = cfg.clone();
    apply_config_value(&mut updated, key, val, paths)?;
    save_config(paths, &updated)?;
    let new_fields = get_config_fields(&updated);
    let new_val = new_fields.iter().find(|f| f.key == key).map(|f| f.value.as_str()).unwrap_or(val);
    if is_default {
        println!("{} = {}  {}", key, styler.bold_white(new_val), styler.dim("(default)"));
    } else {
        println!("{} = {}  {}", key, styler.bold_white(new_val), styler.dim(&std::format!("(was {})", old_val)));
    }
    Ok(())
}

fn config_reset(paths: &LaciePaths) -> Result<(), DiagError> {
    let q = locale::t("cmd_config.reset_prompt");
    if !prompt_yes_no(&q, false, "config reset")? {
        return Ok(());
    }
    let default_cfg = Config {
        init: true,
        ..Default::default()
    };
    save_config(paths, &default_cfg)
}

pub fn execute_config(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let styler = Styler::new(cfg.colors.parse().unwrap_or(ColorMode::Terminal));
    if args.is_empty() {
        return config_list(cfg, &styler);
    }
    let sub = &args[0];
    match sub.as_str() {
        "list" => config_list(cfg, &styler),
        "get" if args.len() == 2 => config_get(cfg, &args[1]),
        "set" if args.len() == 3 => config_set(paths, cfg, &args[1], &args[2], &styler),
        "reset" => config_reset(paths),
        _ => Err(DiagError::from_code("E017", &[("expected", "list, get {key}, set {key} {val}, reset")])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_get_and_set() {
        let tmp = std::env::temp_dir().join("lacie_test_config");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let cfg = Config::default();

        assert!(execute_config(&paths, &cfg, &["list".to_string()]).is_ok());
        assert!(execute_config(&paths, &cfg, &["get".to_string(), "language".to_string()]).is_ok());
        assert!(execute_config(&paths, &cfg, &["get".to_string(), "unknown".to_string()]).is_err());
    }
}
