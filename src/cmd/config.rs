use crate::config::paths::LaciePaths;
use crate::config::{default_linux_priority, save_config, Config};
use crate::diag::DiagError;
use crate::locale;
use crate::platform::{current_os, OsType};
use crate::state::sources::SourcesDatabase;
use crate::ui::color::{ColorMode, Styler};
use crate::ui::prompt::prompt_yes_no;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigSection {
    Ui,
    Behavior,
    Engine,
}

impl ConfigSection {
    fn title_key(&self) -> &'static str {
        match self {
            ConfigSection::Ui => "cmd_config.sec_ui",
            ConfigSection::Behavior => "cmd_config.sec_behavior",
            ConfigSection::Engine => "cmd_config.sec_engine",
        }
    }
}

struct ConfigField {
    pub key: &'static str,
    pub value: String,
    pub default: String,
    pub section: ConfigSection,
    pub hint_key: &'static str,
}

fn get_config_fields(cfg: &Config) -> Vec<ConfigField> {
    let os = current_os();
    let mut fields = vec![
        ConfigField {
            key: "language",
            value: cfg.language.clone(),
            default: "auto".to_string(),
            section: ConfigSection::Ui,
            hint_key: "cmd_config.hint_language",
        },
        ConfigField {
            key: "colors",
            value: cfg.colors.clone(),
            default: "terminal".to_string(),
            section: ConfigSection::Ui,
            hint_key: "cmd_config.hint_colors",
        },
        ConfigField {
            key: "verify_sha256",
            value: cfg.verify_sha256.to_string(),
            default: "true".to_string(),
            section: ConfigSection::Behavior,
            hint_key: "cmd_config.hint_verify",
        },
        ConfigField {
            key: "auto_switch",
            value: cfg.auto_switch.to_string(),
            default: "true".to_string(),
            section: ConfigSection::Behavior,
            hint_key: "cmd_config.hint_switch",
        },
        ConfigField {
            key: "confirm",
            value: cfg.confirm.to_string(),
            default: "true".to_string(),
            section: ConfigSection::Behavior,
            hint_key: "cmd_config.hint_confirm",
        },
        ConfigField {
            key: "always_show",
            value: cfg.always_show.to_string(),
            default: "false".to_string(),
            section: ConfigSection::Behavior,
            hint_key: "cmd_config.hint_always_show",
        },
        ConfigField {
            key: "cache_ttl",
            value: cfg.cache_ttl.to_string(),
            default: "10".to_string(),
            section: ConfigSection::Engine,
            hint_key: "cmd_config.hint_cache_ttl",
        },
        ConfigField {
            key: "default_source",
            value: cfg.default_source.clone(),
            default: "github".to_string(),
            section: ConfigSection::Engine,
            hint_key: "cmd_config.hint_source",
        },
    ];

    if os == OsType::Windows {
        fields.push(ConfigField {
            key: "windows_priority",
            value: cfg.windows_priority.clone(),
            default: "msvc".to_string(),
            section: ConfigSection::Engine,
            hint_key: "cmd_config.hint_windows_priority",
        });
    } else if os == OsType::Linux {
        fields.push(ConfigField {
            key: "linux_priority",
            value: cfg.linux_priority.clone(),
            default: default_linux_priority(),
            section: ConfigSection::Engine,
            hint_key: "cmd_config.hint_linux_priority",
        });
    }

    fields
}

fn format_path_for_display(path: &Path) -> String {
    let path_str = path.display().to_string();
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
        && path_str.starts_with(&home)
    {
        return path_str.replacen(&home, "~", 1);
    }
    path_str
}

fn print_section(
    sec: ConfigSection,
    fields: &[ConfigField],
    styler: &Styler,
) -> bool {
    let sec_fields: Vec<&ConfigField> = fields.iter().filter(|f| f.section == sec).collect();
    if sec_fields.is_empty() {
        return false;
    }
    println!("\n  {}", styler.bold_cyan(&locale::t(sec.title_key())));
    let mut any_diff = false;
    for f in sec_fields {
        let is_diff = f.value != f.default;
        if is_diff {
            any_diff = true;
        }
        let marker = if is_diff {
            std::format!("{} ", styler.bold_cyan("●"))
        } else {
            "  ".to_string()
        };
        let val_colored = if is_diff {
            styler.bold_green(&f.value)
        } else {
            f.value.clone()
        };
        let def_colored = styler.dim(&f.default);
        let hint_text = styler.dim(&std::format!("({})", locale::t(f.hint_key)));

        let pad_key = " ".repeat(18usize.saturating_sub(f.key.len()));
        let pad_val = " ".repeat(12usize.saturating_sub(f.value.len()));
        let pad_def = " ".repeat(12usize.saturating_sub(f.default.len()));

        println!(
            "  {} {}{}{}{}{}{}{}{}",
            styler.dim("│"),
            marker,
            styler.bold_white(f.key),
            pad_key,
            val_colored,
            pad_val,
            def_colored,
            pad_def,
            hint_text
        );
    }
    any_diff
}

fn config_list(paths: &LaciePaths, cfg: &Config, styler: &Styler) -> Result<(), DiagError> {
    let fields = get_config_fields(cfg);
    let display_path = format_path_for_display(&paths.config_file);
    let header = locale::format("cmd_config.header", &[("path", &display_path)]);
    println!("\n{} {}", styler.bold_cyan("::"), styler.bold_white(&header));

    let diff_ui = print_section(ConfigSection::Ui, &fields, styler);
    let diff_beh = print_section(ConfigSection::Behavior, &fields, styler);
    let diff_eng = print_section(ConfigSection::Engine, &fields, styler);

    if diff_ui || diff_beh || diff_eng {
        println!("\n  {} {}", styler.bold_cyan("●"), styler.dim(&locale::t("cmd_config.differs_legend")));
    }
    Ok(())
}

fn config_get(cfg: &Config, key: &str) -> Result<(), DiagError> {
    let fields = get_config_fields(cfg);
    if let Some(f) = fields.iter().find(|f| f.key == key) {
        println!("{}", f.value);
        return Ok(());
    }
    if key == "windows_priority" {
        println!("{}", cfg.windows_priority);
        return Ok(());
    }
    if key == "linux_priority" {
        println!("{}", cfg.linux_priority);
        return Ok(());
    }
    Err(DiagError::from_code("E003", &[("key", key)]))
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
        "linux_priority" => {
            let def = default_linux_priority();
            let v = if val == "DEFAULT" { &def } else { val };
            let norm = match v {
                "glibc" | "gnu" => "glibc",
                "musl" => "musl",
                _ => {
                    return Err(DiagError::from_code(
                        "E004",
                        &[("key", key), ("value", val), ("allowed", "glibc, musl")],
                    ));
                }
            };
            cfg.linux_priority = norm.to_string();
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
        return config_list(paths, cfg, &styler);
    }
    let sub = &args[0];
    match sub.as_str() {
        "list" => config_list(paths, cfg, &styler),
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

    #[test]
    fn test_platform_priority_visibility() {
        let cfg = Config::default();
        let fields = get_config_fields(&cfg);
        let os = current_os();

        if os == OsType::Windows {
            assert!(fields.iter().any(|f| f.key == "windows_priority"));
            assert!(!fields.iter().any(|f| f.key == "linux_priority"));
        } else if os == OsType::Linux {
            assert!(fields.iter().any(|f| f.key == "linux_priority"));
            assert!(!fields.iter().any(|f| f.key == "windows_priority"));
        } else {
            assert!(!fields.iter().any(|f| f.key == "windows_priority"));
            assert!(!fields.iter().any(|f| f.key == "linux_priority"));
        }
    }

    #[test]
    fn test_linux_priority_set() {
        let tmp = std::env::temp_dir().join("lacie_test_linux_pri");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        let mut cfg = Config::default();

        assert!(apply_config_value(&mut cfg, "linux_priority", "musl", &paths).is_ok());
        assert_eq!(cfg.linux_priority, "musl");

        assert!(apply_config_value(&mut cfg, "linux_priority", "glibc", &paths).is_ok());
        assert_eq!(cfg.linux_priority, "glibc");

        assert!(apply_config_value(&mut cfg, "linux_priority", "gnu", &paths).is_ok());
        assert_eq!(cfg.linux_priority, "glibc");

        assert!(apply_config_value(&mut cfg, "linux_priority", "invalid", &paths).is_err());
    }
}
