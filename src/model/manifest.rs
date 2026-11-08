use crate::diag::DiagError;
use crate::model::id::is_valid_package_id;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppDependency {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LibDependency {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinaryEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, rename = "static", skip_serializing_if = "Option::is_none")]
    pub is_static: Option<bool>,
}

impl BinaryEntry {
    pub fn validate(&self, target: &str) -> Result<(), DiagError> {
        let count = [self.file.is_some(), self.links.is_some(), self.source.is_some()]
            .iter()
            .filter(|&&b| b)
            .count();
        if count != 1 {
            let reason = std::format!("target `{}` must specify exactly one of file, links, or source", target);
            return Err(DiagError::from_code("E028", &[("file", "lacie.toml"), ("ref", target), ("reason", &reason)]));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceDefaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_number: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apps: Option<Vec<AppDependency>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libs: Option<Vec<LibDependency>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_deps: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prebuild: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postbuild: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preremove: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postremove: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageDeclaration {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_number: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apps: Option<Vec<AppDependency>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libs: Option<Vec<LibDependency>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_deps: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prebuild: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postbuild: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preremove: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postremove: Option<String>,
    #[serde(default)]
    pub binaries: BTreeMap<String, BinaryEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceManifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceDefaults>,
    #[serde(default)]
    pub packages: BTreeMap<String, PackageDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub version_number: u64,
    pub author: Option<String>,
    pub source_code: Option<String>,
    pub links: Vec<String>,
    pub required: Vec<String>,
    pub apps: Vec<AppDependency>,
    pub libs: Vec<LibDependency>,
    pub check_deps: Option<String>,
    pub prebuild: Option<String>,
    pub postbuild: Option<String>,
    pub preremove: Option<String>,
    pub postremove: Option<String>,
    pub binaries: BTreeMap<String, BinaryEntry>,
}

impl WorkspaceManifest {
    pub fn parse(toml_str: &str) -> Result<Self, DiagError> {
        toml::from_str::<Self>(toml_str)
            .map_err(|e| DiagError::from_code("E028", &[("file", "lacie.toml"), ("ref", "manifest"), ("reason", &e.to_string())]))
    }

    pub fn resolve_package(&self, id: &str) -> Result<ResolvedPackage, DiagError> {
        if !is_valid_package_id(id) {
            let reason = std::format!("invalid domain package id `{}`", id);
            return Err(DiagError::from_code("E028", &[("file", "lacie.toml"), ("ref", id), ("reason", &reason)]));
        }
        let pkg = self.packages.get(id)
            .ok_or_else(|| DiagError::from_code("E009", &[("name", id)]))?;
        let ws = self.workspace.as_ref();
        let version = pkg.version.clone()
            .or_else(|| ws.and_then(|w| w.version.clone()))
            .ok_or_else(|| DiagError::from_code("E028", &[("file", "lacie.toml"), ("ref", id), ("reason", "missing version")]))?;
        let version_number = pkg.version_number
            .or_else(|| ws.and_then(|w| w.version_number))
            .unwrap_or(1);
        if pkg.binaries.is_empty() {
            return Err(DiagError::from_code("E028", &[("file", "lacie.toml"), ("ref", id), ("reason", "no binaries defined")]));
        }
        let mut req_list = Vec::new();
        if let Some(r) = pkg.required.as_ref().or(pkg.requires.as_ref()) {
            req_list.extend(r.clone());
        } else if let Some(r) = ws.and_then(|w| w.required.as_ref().or(w.requires.as_ref())) {
            req_list.extend(r.clone());
        }
        for (target, bin) in &pkg.binaries {
            bin.validate(target)?;
        }
        Ok(ResolvedPackage {
            id: id.to_string(),
            name: pkg.name.clone(),
            version,
            version_number,
            author: pkg.author.clone().or_else(|| ws.and_then(|w| w.author.clone())),
            source_code: pkg.source_code.clone().or_else(|| ws.and_then(|w| w.source_code.clone())),
            links: pkg.links.clone().or_else(|| ws.and_then(|w| w.links.clone())).unwrap_or_default(),
            required: req_list,
            apps: pkg.apps.clone().or_else(|| ws.and_then(|w| w.apps.clone())).unwrap_or_default(),
            libs: pkg.libs.clone().or_else(|| ws.and_then(|w| w.libs.clone())).unwrap_or_default(),
            check_deps: pkg.check_deps.clone().or_else(|| ws.and_then(|w| w.check_deps.clone())),
            prebuild: pkg.prebuild.clone().or_else(|| ws.and_then(|w| w.prebuild.clone())),
            postbuild: pkg.postbuild.clone().or_else(|| ws.and_then(|w| w.postbuild.clone())),
            preremove: pkg.preremove.clone().or_else(|| ws.and_then(|w| w.preremove.clone())),
            postremove: pkg.postremove.clone().or_else(|| ws.and_then(|w| w.postremove.clone())),
            binaries: pkg.binaries.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_workspace_inheritance() {
        let toml_data = r#"
[workspace]
author = "shareui"
version = "1.0.0"
version_number = 1
source_code = "https://github.com/shareui/lacie"
links = ["https://example.com"]

[packages."com.example.app"]
name = "app"
apps = [
  { id = "org.other.tool", from = "github:shareui/tool", min_number = 3 },
]
libs = [
  { id = "org.other.libfoo", from = "github:shareui/libfoo", dest = "lib" },
]
check_deps = "check_deps.rb"
prebuild = "prebuild.rb"
postbuild = "postbuild.rb"

[packages."com.example.app".binaries."linux-gnu-x86_64"]
file = "app-linux-gnu-x86_64"
size = 3586129
sha256 = "3f2a9c0e41b7d2a85c6f0e19b4d7a3c28e5f1096ab7d4c3e2f81b0a9d6c57e43"

[packages."com.example.tool"]
name = "tool"
version = "2.3.1"
version_number = 12

[packages."com.example.tool".binaries."windows-msvc-x86_64"]
links = ["https://example.com/tool.exe"]
"#;
        let manifest = WorkspaceManifest::parse(toml_data).unwrap();
        let app = manifest.resolve_package("com.example.app").unwrap();
        assert_eq!(app.name, "app");
        assert_eq!(app.version, "1.0.0");
        assert_eq!(app.author, Some("shareui".to_string()));
        assert_eq!(app.apps.len(), 1);
        assert_eq!(app.libs.len(), 1);
        assert_eq!(app.binaries.len(), 1);
        assert_eq!(app.binaries.get("linux-gnu-x86_64").and_then(|b| b.size), Some(3586129));

        let tool = manifest.resolve_package("com.example.tool").unwrap();
        assert_eq!(tool.name, "tool");
        assert_eq!(tool.version, "2.3.1");
        assert_eq!(tool.version_number, 12);
        assert_eq!(tool.author, Some("shareui".to_string()));
    }
}
