use crate::diag::DiagError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockSourceEntry {
    pub name: String,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockPackageEntry {
    pub id: String,
    #[serde(rename = "ref")]
    pub package_ref: String,
    pub release: String,
    pub version: String,
    pub target: String,
    pub sha256: String,
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockFile {
    pub format: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<LockSourceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<LockPackageEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, toml::Value>>,
}

impl Default for LockFile {
    fn default() -> Self {
        Self {
            format: 1,
            sources: Vec::new(),
            packages: Vec::new(),
            config: None,
        }
    }
}

impl LockFile {
    pub fn parse(toml_str: &str) -> Result<Self, DiagError> {
        let lock: LockFile = toml::from_str(toml_str)
            .map_err(|e| DiagError::from_code("E028", &[("file", "lacie-lock.toml"), ("ref", "lock"), ("reason", &e.to_string())]))?;
        if lock.format != 1 {
            let reason = std::format!("unsupported lock format version {}", lock.format);
            return Err(DiagError::from_code("E028", &[("file", "lacie-lock.toml"), ("ref", "lock"), ("reason", &reason)]));
        }
        Ok(lock)
    }

    pub fn to_toml_string(&self) -> Result<String, DiagError> {
        toml::to_string_pretty(self)
            .map_err(|e| DiagError::new("E028", e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_file_roundtrip() {
        let mut lock = LockFile::default();
        lock.sources.push(LockSourceEntry {
            name: "example".to_string(),
            origin: "https://example.com/source.toml".to_string(),
        });
        let serialized = lock.to_toml_string().unwrap();
        let parsed = LockFile::parse(&serialized).unwrap();
        assert_eq!(lock.sources, parsed.sources);
    }
}
