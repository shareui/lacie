use crate::diag::DiagError;
use crate::state::atomic_write_toml;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub version_number: u64,
    pub origin: String,
    #[serde(default)]
    pub builtin: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourcesDatabase {
    #[serde(default)]
    pub sources: BTreeMap<String, SourceEntry>,
}

impl SourcesDatabase {
    pub fn load(file: &Path) -> Result<Self, DiagError> {
        let mut db = if !file.exists() {
            Self::default()
        } else {
            let content = std::fs::read_to_string(file)
                .map_err(|e| DiagError::new("E015", e.to_string()))?;
            if content.trim().is_empty() {
                Self::default()
            } else {
                toml::from_str::<Self>(&content)
                    .map_err(|e| DiagError::new("E028", e.to_string()))?
            }
        };
        if !db.sources.contains_key("github") {
            db.sources.insert(
                "github".to_string(),
                SourceEntry {
                    id: "lacie.github".to_string(),
                    name: "github".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    version_number: 1,
                    origin: "builtin".to_string(),
                    builtin: true,
                    enabled: true,
                    added_at: None,
                },
            );
        }
        Ok(db)
    }

    pub fn save(&self, target_file: &Path, var_dir: &Path) -> Result<(), DiagError> {
        atomic_write_toml(target_file, var_dir, self)
    }

    pub fn get(&self, name: &str) -> Option<&SourceEntry> {
        self.sources.get(name)
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.sources.get(name).map(|s| s.enabled).unwrap_or(false)
    }

    pub fn find_by_name_or_id(&self, query: &str) -> Result<String, DiagError> {
        let mut matches = Vec::new();
        for (name, entry) in &self.sources {
            if name == query || entry.id == query {
                matches.push(name.clone());
            }
        }
        if matches.len() > 1 {
            return Err(DiagError::from_code("E012", &[("name", query), ("ids", &matches.join(", "))]));
        }
        if let Some(name) = matches.first() {
            return Ok(name.clone());
        }
        Err(DiagError::from_code("E008", &[("source", query)]))
    }

    pub fn add(&mut self, entry: SourceEntry) -> Result<(), DiagError> {
        if self.sources.contains_key(&entry.name) {
            return Err(DiagError::from_code("E030", &[("name", &entry.name)]));
        }
        self.sources.insert(entry.name.clone(), entry);
        Ok(())
    }

    pub fn disable(&mut self, name: &str, default_source: &str) -> Result<(), DiagError> {
        if name == default_source {
            return Err(DiagError::from_code("E032", &[("name", name)]));
        }
        let entry = self.sources.get_mut(name)
            .ok_or_else(|| DiagError::from_code("E008", &[("source", name)]))?;
        entry.enabled = false;
        Ok(())
    }

    pub fn enable(&mut self, name: &str) -> Result<(), DiagError> {
        let entry = self.sources.get_mut(name)
            .ok_or_else(|| DiagError::from_code("E008", &[("source", name)]))?;
        entry.enabled = true;
        Ok(())
    }

    pub fn remove(&mut self, name: &str, default_source: &str) -> Result<(), DiagError> {
        let entry = self.sources.get(name)
            .ok_or_else(|| DiagError::from_code("E008", &[("source", name)]))?;
        if entry.builtin {
            return Err(DiagError::from_code("E031", &[("name", name)]));
        }
        if name == default_source {
            return Err(DiagError::from_code("E032", &[("name", name)]));
        }
        self.sources.remove(name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sources_database() {
        let mut db = SourcesDatabase::default();
        let src = SourceEntry {
            id: "lacie.github".to_string(),
            name: "github".to_string(),
            version: "0.1.0".to_string(),
            version_number: 1,
            origin: "builtin".to_string(),
            builtin: true,
            enabled: true,
            added_at: None,
        };
        db.add(src).unwrap();
        assert!(db.remove("github", "other").is_err());
        assert!(db.disable("github", "github").is_err());
    }
}
