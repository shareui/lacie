use crate::diag::DiagError;
use std::path::Path;

pub const GITHUB_SOURCE_TOML: &str = include_str!("../../plugins/github/source.toml");
pub const GITHUB_RB: &str = include_str!("../../plugins/github/github.rb");

pub fn materialize_builtin_github(plugins_dir: &Path) -> Result<(), DiagError> {
    let github_dir = plugins_dir.join("github");
    std::fs::create_dir_all(&github_dir)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let toml_path = github_dir.join("source.toml");
    let rb_path = github_dir.join("github.rb");
    if !toml_path.exists() {
        std::fs::write(&toml_path, GITHUB_SOURCE_TOML)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
    }
    let rb_needs_update = !rb_path.exists()
        || std::fs::read_to_string(&rb_path).map(|s| s != GITHUB_RB).unwrap_or(false);
    if rb_needs_update {
        std::fs::write(&rb_path, GITHUB_RB)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&rb_path, std::fs::Permissions::from_mode(0o755));
        }
    }
    Ok(())
}
