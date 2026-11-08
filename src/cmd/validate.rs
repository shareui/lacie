use crate::diag::{print_warning, DiagError, DiagWarning};
use crate::model::id::is_valid_package_id;
use crate::model::manifest::WorkspaceManifest;
use crate::model::source_toml::SourceManifest;
use crate::net::hash::compute_sha256_file;
use std::path::{Path, PathBuf};

fn validate_manifest_files(
    manifest: &WorkspaceManifest,
    base_dir: &Path,
    warnings: &mut usize,
    errors: &mut usize,
) {
    for (id, pkg) in &manifest.packages {
        for (target, bin) in &pkg.binaries {
            if let Some(ref filename) = bin.file {
                let local_path = base_dir.join(filename);
                if local_path.is_file() {
                    if let Some(ref exp_sha) = bin.sha256
                        && let Ok(actual_sha) = compute_sha256_file(&local_path)
                        && actual_sha.to_lowercase() != exp_sha.to_lowercase()
                    {
                        eprintln!("Error [028]: Invalid lacie.toml: binary {} of {} sha256 mismatch", target, id);
                        *errors += 1;
                    }
                } else {
                    print_warning(&DiagWarning::from_code("W006", &[("file", filename), ("path", &base_dir.display().to_string())]));
                    *warnings += 1;
                }
            }
        }
    }
}

pub fn execute_validate(args: &[String]) -> Result<(), DiagError> {
    let target_file = args.iter().find(|a| !a.starts_with('-')).cloned().unwrap_or_else(|| "./lacie.toml".to_string());
    let path = PathBuf::from(target_file.strip_prefix("file://").unwrap_or(&target_file));
    if !path.is_file() {
        return Err(DiagError::from_code("E028", &[("file", &target_file), ("ref", &target_file), ("reason", "file not found")]));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;

    let mut errors = 0;
    let mut warnings = 0;
    let base_dir = path.parent().unwrap_or(Path::new("."));

    if content.contains("entry =") {
        if let Err(e) = SourceManifest::parse(&content) {
            eprintln!("Error [028]: Invalid source.toml in {}: {}", target_file, e.message);
            errors += 1;
        }
    } else {
        match WorkspaceManifest::parse(&content) {
            Ok(manifest) => {
                for id in manifest.packages.keys() {
                    if !is_valid_package_id(id) {
                        eprintln!("Error [028]: Invalid lacie.toml: invalid domain id `{}`", id);
                        errors += 1;
                    }
                }
                validate_manifest_files(&manifest, base_dir, &mut warnings, &mut errors);
            }
            Err(e) => {
                eprintln!("Error [028]: Invalid lacie.toml in {}: {}", target_file, e.message);
                errors += 1;
            }
        }
    }

    if errors > 0 || warnings > 0 {
        let err_word = if errors == 1 { "error" } else { "errors" };
        let warn_word = if warnings == 1 { "warning" } else { "warnings" };
        println!("\n{} {}, {} {}", errors, err_word, warnings, warn_word);
    }
    if errors > 0 {
        return Err(DiagError::new("E028", "validation failed with errors".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_manifest() {
        let tmp = std::env::temp_dir().join("lacie_test_validate.toml");
        std::fs::write(&tmp, r#"
[workspace]
author = "shareui"
version = "0.1.0"
version_number = 1

[packages."com.example.app"]
name = "app"

[packages."com.example.app".binaries."linux-gnu-x86_64"]
file = "app"
"#).unwrap();
        let _ = execute_validate(&[tmp.to_string_lossy().to_string()]);
    }
}
