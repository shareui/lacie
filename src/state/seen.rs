use crate::diag::DiagError;
use crate::state::atomic_write_toml;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeenDatabase {
    #[serde(default)]
    pub manifests: BTreeMap<String, DateTime<Utc>>,
    #[serde(default)]
    pub scripts: BTreeMap<String, DateTime<Utc>>,
}

impl SeenDatabase {
    pub fn load(file: &Path) -> Result<Self, DiagError> {
        if !file.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(file)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        if content.trim().is_empty() {
            return Ok(Self::default());
        }
        toml::from_str::<Self>(&content)
            .map_err(|e| DiagError::new("E028", e.to_string()))
    }

    pub fn save(&self, target_file: &Path, var_dir: &Path) -> Result<(), DiagError> {
        atomic_write_toml(target_file, var_dir, self)
    }

    pub fn is_manifest_seen(&self, sha256: &str) -> bool {
        self.manifests.contains_key(sha256)
    }

    pub fn is_script_seen(&self, sha256: &str) -> bool {
        self.scripts.contains_key(sha256)
    }

    pub fn add_manifest(&mut self, sha256: &str) {
        self.manifests.insert(sha256.to_string(), Utc::now());
    }

    pub fn add_script(&mut self, sha256: &str) {
        self.scripts.insert(sha256.to_string(), Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seen_database() {
        let mut db = SeenDatabase::default();
        let hash = "3f2a9c0e41b7d2a85c6f0e19b4d7a3c28e5f1096ab7d4c3e2f81b0a9d6c57e43";
        assert!(!db.is_manifest_seen(hash));
        db.add_manifest(hash);
        assert!(db.is_manifest_seen(hash));
    }
}
