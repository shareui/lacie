use crate::diag::DiagError;
use crate::model::source_toml::SourceManifest;
use crate::net::hash::compute_sha256_bytes;
use crate::ruby::builtin::materialize_builtin_github;
use crate::ruby::rpc::RpcClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag: String,
    #[serde(default)]
    pub pre_release: bool,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub has_manifest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub storage: String,
    #[serde(default)]
    pub stars: u64,
}

#[derive(Serialize, Deserialize)]
struct CacheWrapper {
    timestamp: u64,
    data: Value,
}

pub struct SourcePlugin {
    pub name: String,
    pub manifest: SourceManifest,
    pub dir: PathBuf,
    client: Option<RpcClient>,
}

impl SourcePlugin {
    pub fn load(name: &str, plugins_dir: &Path) -> Result<Self, DiagError> {
        if name == "github" {
            materialize_builtin_github(plugins_dir)?;
        }
        let dir = plugins_dir.join(name);
        let manifest_path = dir.join("source.toml");
        if !manifest_path.is_file() {
            return Err(DiagError::from_code("E008", &[("source", name)]));
        }
        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        let manifest = SourceManifest::parse(&content)?;
        Ok(Self {
            name: name.to_string(),
            manifest,
            dir,
            client: None,
        })
    }

    fn get_client(&mut self) -> Result<&mut RpcClient, DiagError> {
        if self.client.is_none() {
            let entry_path = self.dir.join(&self.manifest.entry);
            let client = RpcClient::new(&self.name, &entry_path, &self.dir)?;
            self.client = Some(client);
        }
        Ok(self.client.as_mut().unwrap())
    }

    pub fn search(
        &mut self,
        query: &str,
        use_cache: bool,
        cache_ttl: u64,
        cache_dir: &Path,
    ) -> Result<Vec<SearchResult>, DiagError> {
        if !self.manifest.capabilities.contains(&"search".to_string()) {
            return Err(DiagError::from_code("E034", &[("name", &self.name)]));
        }
        let val = self.cached_call("search", json!({"query": query}), query, use_cache, cache_ttl, cache_dir)?;
        serde_json::from_value(val)
            .map_err(|e| DiagError::from_code("E033", &[("name", &self.name), ("reason", &e.to_string())]))
    }

    pub fn releases(
        &mut self,
        storage: &str,
        use_cache: bool,
        cache_ttl: u64,
        cache_dir: &Path,
    ) -> Result<Vec<ReleaseInfo>, DiagError> {
        let val = self.cached_call("releases", json!({"storage": storage}), storage, use_cache, cache_ttl, cache_dir)?;
        serde_json::from_value(val)
            .map_err(|e| DiagError::from_code("E033", &[("name", &self.name), ("reason", &e.to_string())]))
    }

    pub fn manifest(&mut self, storage: &str, release: &str) -> Result<String, DiagError> {
        let client = self.get_client()?;
        let res = client.call("manifest", json!({"storage": storage, "release": release}))?;
        res.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| DiagError::from_code("E033", &[("name", &self.name), ("reason", "invalid manifest text")]))
    }

    pub fn fetch(&mut self, storage: &str, release: &str, file: &str, dest: &Path) -> Result<(), DiagError> {
        let dest_str = dest.to_string_lossy();
        let client = self.get_client()?;
        client.call("fetch", json!({
            "storage": storage,
            "release": release,
            "file": file,
            "dest": dest_str,
        }))?;
        Ok(())
    }

    fn cached_call(
        &mut self,
        method: &str,
        params: Value,
        cache_key_arg: &str,
        use_cache: bool,
        cache_ttl: u64,
        cache_dir: &Path,
    ) -> Result<Value, DiagError> {
        let key = std::format!("{}:{}:{}", self.name, method, cache_key_arg);
        let hash = compute_sha256_bytes(key.as_bytes());
        let cache_file = cache_dir.join(std::format!("{}_{}.json", self.name, &hash[..16]));

        if use_cache
            && cache_ttl > 0
            && cache_file.is_file()
            && let Some(val) = read_cache_file(&cache_file, cache_ttl)
        {
            return Ok(val);
        }
        let client = self.get_client()?;
        let res = client.call(method, params)?;
        if cache_ttl > 0 {
            let _ = write_cache_file(&cache_file, &res);
        }
        Ok(res)
    }
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_cache_file(path: &Path, cache_ttl_minutes: u64) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    let wrapper: CacheWrapper = serde_json::from_str(&content).ok()?;
    let now = current_epoch_seconds();
    if now.saturating_sub(wrapper.timestamp) < cache_ttl_minutes * 60 {
        Some(wrapper.data)
    } else {
        None
    }
}

fn write_cache_file(path: &Path, data: &Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let wrapper = CacheWrapper {
        timestamp: current_epoch_seconds(),
        data: data.clone(),
    };
    let content = serde_json::to_string(&wrapper)?;
    std::fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_plugin_materialize_and_load() {
        let tmp = std::env::temp_dir().join("lacie_test_gh_plugin");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let mut plugin = SourcePlugin::load("github", &tmp).unwrap();
        assert_eq!(plugin.manifest.name, "github");
        assert_eq!(plugin.manifest.api, 1);
        let client = plugin.get_client();
        assert!(client.is_ok());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
