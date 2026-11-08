pub mod paths;
pub mod shell;

use crate::diag::DiagError;
use paths::LaciePaths;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub init: bool,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_colors")]
    pub colors: String,
    #[serde(default = "default_true")]
    pub verify_sha256: bool,
    #[serde(default = "default_true")]
    pub auto_switch: bool,
    #[serde(default = "default_true")]
    pub confirm: bool,
    #[serde(default = "default_false")]
    pub always_show: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
    #[serde(default = "default_source")]
    pub default_source: String,
    #[serde(default = "default_windows_priority")]
    pub windows_priority: String,
}

fn default_language() -> String {
    "auto".to_string()
}

fn default_colors() -> String {
    "terminal".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_cache_ttl() -> u64 {
    10
}

fn default_source() -> String {
    "github".to_string()
}

fn default_windows_priority() -> String {
    "msvc".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init: false,
            language: default_language(),
            colors: default_colors(),
            verify_sha256: default_true(),
            auto_switch: default_true(),
            confirm: default_true(),
            always_show: default_false(),
            cache_ttl: default_cache_ttl(),
            default_source: default_source(),
            windows_priority: default_windows_priority(),
        }
    }
}

pub fn check_init(paths: &LaciePaths) -> bool {
    if !paths.config_file.is_file() {
        return false;
    }
    let content = match std::fs::read_to_string(&paths.config_file) {
        Ok(c) => c,
        Err(_) => return false,
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("init") {
            let parts: Vec<&str> = trimmed.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 && parts[0] == "init" && parts[1] == "true" {
                return true;
            }
        }
    }
    false
}

pub fn load_config(paths: &LaciePaths) -> Result<Config, DiagError> {
    if !check_init(paths) {
        return Err(DiagError::from_code("E001", &[]));
    }
    let content = std::fs::read_to_string(&paths.config_file)
        .map_err(|e| DiagError::from_code("E028", &[("file", "config.toml"), ("ref", "config"), ("reason", &e.to_string())]))?;
    toml::from_str::<Config>(&content)
        .map_err(|e| DiagError::from_code("E028", &[("file", "config.toml"), ("ref", "config"), ("reason", &e.to_string())]))
}

pub fn save_config(paths: &LaciePaths, config: &Config) -> Result<(), DiagError> {
    let serialized = toml::to_string_pretty(config)
        .map_err(|e| DiagError::from_code("E028", &[("file", "config.toml"), ("ref", "config"), ("reason", &e.to_string())]))?;
    paths.ensure_all_dirs()
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let tmp_path = paths.var_dir.join("config.tmp");
    let mut file = std::fs::File::create(&tmp_path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    file.flush()
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    drop(file);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp_path, &paths.config_file)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_init_flag() {
        let tmp = std::env::temp_dir().join("lacie_test_check_init");
        let p = LaciePaths::from_root(tmp.clone());
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(!check_init(&p));

        p.ensure_all_dirs().unwrap();
        std::fs::write(&p.config_file, "init = false\nlanguage = \"en\"\n").unwrap();
        assert!(!check_init(&p));

        std::fs::write(&p.config_file, "init = true\nlanguage = \"en\"\n").unwrap();
        assert!(check_init(&p));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
