use std::path::Path;

pub fn set_binary_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    let _ = path;
    Ok(())
}

pub fn set_file_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }
    let _ = path;
    Ok(())
}

pub fn set_dir_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(path, perms)?;
    }
    let _ = path;
    Ok(())
}

pub fn activate_binary(bin_path: &Path, active_path: &Path) -> std::io::Result<()> {
    if active_path.exists() || active_path.is_symlink() {
        std::fs::remove_file(active_path)?;
    }
    if let Some(parent) = active_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(bin_path, active_path)?;
    }
    #[cfg(windows)]
    {
        std::fs::rename(bin_path, active_path)?;
    }
    Ok(())
}

pub fn deactivate_binary(active_path: &Path, bin_path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        if active_path.exists() || active_path.is_symlink() {
            std::fs::remove_file(active_path)?;
        }
        let _ = bin_path;
    }
    #[cfg(windows)]
    {
        if active_path.exists() {
            if let Some(parent) = bin_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(active_path, bin_path)?;
        }
    }
    Ok(())
}
