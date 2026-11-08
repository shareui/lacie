#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAddRef {
    pub source: String,
    pub storage: String,
    pub packages: Vec<String>,
    pub release: Option<String>,
    pub is_local_file: bool,
    pub is_direct_url: bool,
}

pub fn parse_add_ref(input: &str, default_source: &str) -> ParsedAddRef {
    let trimmed = input.trim();
    let mut parts = trimmed.split_whitespace();
    let main_ref = parts.next().unwrap_or(trimmed);
    let release = parts.next().map(|s| s.to_string());

    if main_ref.starts_with("http://") || main_ref.starts_with("https://") {
        return ParsedAddRef {
            source: "url".to_string(),
            storage: main_ref.to_string(),
            packages: Vec::new(),
            release,
            is_local_file: false,
            is_direct_url: true,
        };
    }
    if main_ref.starts_with("file://") || main_ref.starts_with("./") || main_ref.starts_with('/') {
        return ParsedAddRef {
            source: "local".to_string(),
            storage: main_ref.to_string(),
            packages: Vec::new(),
            release,
            is_local_file: true,
            is_direct_url: false,
        };
    }

    let (source_and_storage, packages) = match main_ref.split_once('#') {
        Some((before, after)) => {
            let pkgs = after.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            (before, pkgs)
        }
        None => (main_ref, Vec::new()),
    };

    let (source, storage) = match source_and_storage.split_once(':') {
        Some((src, stor)) => (src.to_string(), stor.to_string()),
        None => (default_source.to_string(), source_and_storage.to_string()),
    };

    ParsedAddRef {
        source,
        storage,
        packages,
        release,
        is_local_file: false,
        is_direct_url: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_add_ref() {
        let p1 = parse_add_ref("github:shareui/lacie#lacie 0.1.0", "github");
        assert_eq!(p1.source, "github");
        assert_eq!(p1.storage, "shareui/lacie");
        assert_eq!(p1.packages, vec!["lacie"]);
        assert_eq!(p1.release, Some("0.1.0".to_string()));

        let p2 = parse_add_ref("shareui/tool", "github");
        assert_eq!(p2.source, "github");
        assert_eq!(p2.storage, "shareui/tool");
        assert!(p2.packages.is_empty());
        assert!(p2.release.is_none());
    }
}
