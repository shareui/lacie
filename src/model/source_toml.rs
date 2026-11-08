use crate::diag::DiagError;
use crate::model::id::is_valid_package_id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginFileEntry {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<String>>,
    #[serde(default)]
    pub executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceManifest {
    pub name: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub version: String,
    #[serde(default = "default_one")]
    pub version_number: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    pub api: u32,
    pub entry: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<PluginFileEntry>,
}

fn default_one() -> u64 {
    1
}

pub fn is_valid_source_name(name: &str) -> bool {
    let len = name.len();
    if !(2..=32).contains(&len) || name.starts_with('-') {
        return false;
    }
    let lower = name.to_ascii_lowercase();
    if matches!(lower.as_str(), "file" | "http" | "https" | "con" | "prn" | "aux" | "nul") {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl SourceManifest {
    pub fn parse(toml_str: &str) -> Result<Self, DiagError> {
        let manifest: SourceManifest = toml::from_str(toml_str)
            .map_err(|e| DiagError::from_code("E028", &[("file", "source.toml"), ("ref", "source"), ("reason", &e.to_string())]))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), DiagError> {
        if self.api != 1 {
            let reason = std::format!("unsupported source plugin api version {}", self.api);
            return Err(DiagError::from_code("E028", &[("file", "source.toml"), ("ref", &self.name), ("reason", &reason)]));
        }
        if !is_valid_source_name(&self.name) {
            let reason = std::format!("invalid source name `{}`", self.name);
            return Err(DiagError::from_code("E028", &[("file", "source.toml"), ("ref", &self.name), ("reason", &reason)]));
        }
        if !is_valid_package_id(&self.id) {
            let reason = std::format!("invalid domain id `{}`", self.id);
            return Err(DiagError::from_code("E028", &[("file", "source.toml"), ("ref", &self.name), ("reason", &reason)]));
        }
        for f in &self.files {
            if f.file.is_some() && f.links.is_some() {
                let reason = std::format!("`{}` specifies both file and links", f.path);
                return Err(DiagError::from_code("E028", &[("file", "source.toml"), ("ref", &f.path), ("reason", &reason)]));
            }
        }
        if !self.files.is_empty() && !self.files.iter().any(|f| f.path == self.entry) {
            let reason = std::format!("entry `{}` is not listed in files", self.entry);
            return Err(DiagError::from_code("E028", &[("file", "source.toml"), ("ref", &self.entry), ("reason", &reason)]));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_toml_validation() {
        let toml_data = r#"
name = "github"
id = "lacie.github"
version = "0.1.0"
api = 1
entry = "github.rb"

[[files]]
path = "github.rb"
"#;
        let manifest = SourceManifest::parse(toml_data).unwrap();
        assert_eq!(manifest.name, "github");
        assert_eq!(manifest.api, 1);
    }
}
