use crate::diag::DiagError;
use crate::state::atomic_write_toml;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptHashes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preremove: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postremove: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptsDatabase {
    #[serde(default, flatten)]
    pub packages: BTreeMap<String, BTreeMap<String, ScriptHashes>>,
}

impl ScriptsDatabase {
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

    pub fn get(&self, id: &str, version: &str) -> Option<&ScriptHashes> {
        self.packages.get(id).and_then(|vers| vers.get(version))
    }

    pub fn set(&mut self, id: &str, version: &str, hashes: ScriptHashes) {
        let pkg = self.packages.entry(id.to_string()).or_default();
        pkg.insert(version.to_string(), hashes);
    }

    pub fn remove(&mut self, id: &str, version: &str) -> bool {
        if let Some(pkg) = self.packages.get_mut(id) {
            let removed = pkg.remove(version).is_some();
            if pkg.is_empty() {
                self.packages.remove(id);
            }
            return removed;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scripts_database() {
        let mut db = ScriptsDatabase::default();
        let hashes = ScriptHashes {
            preremove: Some("hash1".to_string()),
            postremove: Some("hash2".to_string()),
        };
        db.set("com.example.app", "0.1.0", hashes.clone());
        assert_eq!(db.get("com.example.app", "0.1.0"), Some(&hashes));
        assert!(db.remove("com.example.app", "0.1.0"));
        assert!(db.get("com.example.app", "0.1.0").is_none());
    }
}
