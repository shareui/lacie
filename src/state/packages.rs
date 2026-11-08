use crate::diag::DiagError;
use crate::state::atomic_write_toml;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageVersionEntry {
    pub version: String,
    #[serde(default)]
    pub version_number: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released_at: Option<DateTime<Utc>>,
    pub source: String,
    pub storage: String,
    pub target: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageEntry {
    pub name: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
    #[serde(default)]
    pub frozen: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_version: Option<String>,
    #[serde(default)]
    pub versions: BTreeMap<String, PackageVersionEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackagesDatabase {
    #[serde(default)]
    pub packages: BTreeMap<String, PackageEntry>,
}

impl PackagesDatabase {
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

    pub fn find_by_id(&self, id: &str) -> Option<&PackageEntry> {
        self.packages.get(id)
    }

    pub fn get_package_mut(&mut self, id: &str) -> Option<&mut PackageEntry> {
        self.packages.get_mut(id)
    }

    pub fn find_by_name(&self, name: &str) -> Vec<(&String, &PackageEntry)> {
        self.packages
            .iter()
            .filter(|(_, pkg)| pkg.name == name || pkg.command == name)
            .collect()
    }

    pub fn insert_version(
        &mut self,
        id: &str,
        name: &str,
        command: Option<&str>,
        version_entry: PackageVersionEntry,
    ) {
        let cmd = command.unwrap_or(name);
        let entry = self.packages.entry(id.to_string()).or_insert_with(|| PackageEntry {
            name: name.to_string(),
            command: cmd.to_string(),
            active: None,
            frozen: false,
            frozen_version: None,
            versions: BTreeMap::new(),
        });
        entry.versions.insert(version_entry.version.clone(), version_entry);
    }

    pub fn remove_version(&mut self, id: &str, version: &str) -> bool {
        if let Some(entry) = self.packages.get_mut(id) {
            entry.versions.remove(version);
            if entry.active.as_deref() == Some(version) {
                entry.active = None;
            }
            if entry.versions.is_empty() {
                self.packages.remove(id);
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packages_database_crud() {
        let mut db = PackagesDatabase::default();
        let ver = PackageVersionEntry {
            version: "0.1.0".to_string(),
            version_number: 1,
            author: Some("shareui".to_string()),
            links: vec![],
            source_code: None,
            installed_at: None,
            released_at: None,
            source: "github".to_string(),
            storage: "shareui/app".to_string(),
            target: "linux-gnu-x86_64".to_string(),
            sha256: "abc123".to_string(),
        };
        db.insert_version("com.example.app", "app", None, ver);
        assert_eq!(db.find_by_name("app").len(), 1);
        assert!(db.find_by_id("com.example.app").is_some());
        assert!(db.remove_version("com.example.app", "0.1.0"));
        assert!(db.find_by_id("com.example.app").is_none());
    }
}
