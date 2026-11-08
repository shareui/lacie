use crate::diag::DiagError;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

pub struct HttpClient {
    client: reqwest::blocking::Client,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("lacie/0.1.0")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub fn download_file_with_progress<F: FnMut(u64)>(
        &self,
        url: &str,
        dest: &Path,
        mut on_progress: F,
    ) -> Result<(), DiagError> {
        use std::io::Read;
        let mut resp = self.client.get(url)
            .send()
            .map_err(|e| DiagError::from_code("E019", &[("url", url), ("reason", &e.to_string())]))?;
        if !resp.status().is_success() {
            let reason = std::format!("HTTP status {}", resp.status());
            return Err(DiagError::from_code("E019", &[("url", url), ("reason", &reason)]));
        }
        let mut file = std::fs::File::create(dest)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        let mut buf = [0u8; 16384];
        let mut downloaded = 0u64;
        loop {
            let n = resp.read(&mut buf)
                .map_err(|e| DiagError::from_code("E019", &[("url", url), ("reason", &e.to_string())]))?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])
                .map_err(|e| DiagError::new("E015", e.to_string()))?;
            downloaded += n as u64;
            on_progress(downloaded);
        }
        file.flush()
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        Ok(())
    }

    pub fn download_file(&self, url: &str, dest: &Path) -> Result<(), DiagError> {
        self.download_file_with_progress(url, dest, |_| {})
    }

    pub fn download_text(&self, url: &str) -> Result<String, DiagError> {
        let resp = self.client.get(url)
            .send()
            .map_err(|e| DiagError::from_code("E019", &[("url", url), ("reason", &e.to_string())]))?;
        if !resp.status().is_success() {
            let reason = std::format!("HTTP status {}", resp.status());
            return Err(DiagError::from_code("E019", &[("url", url), ("reason", &reason)]));
        }
        resp.text().map_err(|e| DiagError::from_code("E019", &[("url", url), ("reason", &e.to_string())]))
    }

    pub fn get_github_asset_size(
        &self,
        file_url: &str,
        session_cache: &mut HashMap<String, Option<u64>>,
    ) -> Option<u64> {
        if let Some(cached) = session_cache.get(file_url) {
            return *cached;
        }
        if !is_github_url(file_url) {
            session_cache.insert(file_url.to_string(), None);
            return None;
        }
        let size = self.query_github_size(file_url);
        session_cache.insert(file_url.to_string(), size);
        size
    }

    fn query_github_size(&self, file_url: &str) -> Option<u64> {
        let (owner, repo, tag, file_name) = parse_github_release_url(file_url)?;
        let api_url = std::format!("https://api.github.com/repos/{}/{}/releases/tags/{}", owner, repo, tag);
        let mut req = self.client.get(&api_url);
        if let Ok(token) = std::env::var("LACIE_GH_TOKEN")
            && !token.trim().is_empty()
        {
            req = req.header("Authorization", std::format!("Bearer {}", token.trim()));
        }
        let resp = req.send().ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let json: serde_json::Value = resp.json().ok()?;
        let assets = json.get("assets")?.as_array()?;
        for asset in assets {
            let name = asset.get("name")?.as_str()?;
            if name == file_name {
                return asset.get("size")?.as_u64();
            }
        }
        None
    }
}

pub fn is_github_url(url: &str) -> bool {
    url.contains("github.com")
        || url.contains("api.github.com")
        || url.contains("objects.githubusercontent.com")
}

pub fn parse_github_release_url(url: &str) -> Option<(&str, &str, &str, &str)> {
    let stripped = url.strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let parts: Vec<&str> = stripped.split('/').collect();
    if parts.len() >= 6 && parts[2] == "releases" && parts[3] == "download" {
        let owner = parts[0];
        let repo = parts[1];
        let tag = parts[4];
        let filename = parts[5];
        return Some((owner, repo, tag, filename));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        let url = "https://github.com/shareui/lacie/releases/download/v0.1.0/app-linux.tar.gz";
        let parsed = parse_github_release_url(url);
        assert_eq!(parsed, Some(("shareui", "lacie", "v0.1.0", "app-linux.tar.gz")));
    }

    #[test]
    fn test_is_github_url() {
        assert!(is_github_url("https://github.com/foo/bar"));
        assert!(!is_github_url("https://example.com/foo/bar"));
    }
}
