pub mod packages;
pub mod scripts;
pub mod seen;
pub mod sources;

use crate::diag::DiagError;
use serde::Serialize;
use std::io::Write;
use std::path::Path;

pub fn atomic_write_toml<T: Serialize>(target_file: &Path, var_dir: &Path, data: &T) -> Result<(), DiagError> {
    let serialized = toml::to_string_pretty(data)
        .map_err(|e| DiagError::new("E028", e.to_string()))?;
    let file_stem = target_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("state");
    let tmp_path = var_dir.join(std::format!("{}.tmp", file_stem));
    let mut file = std::fs::File::create(&tmp_path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    file.flush()
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    drop(file);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp_path, target_file)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    Ok(())
}
