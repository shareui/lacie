use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaciePaths {
    pub root: PathBuf,
    pub config_file: PathBuf,
    pub packages_file: PathBuf,
    pub sources_file: PathBuf,
    pub seen_file: PathBuf,
    pub scripts_file: PathBuf,
    pub active_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub var_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl LaciePaths {
    pub fn from_root(root: PathBuf) -> Self {
        let var_dir = root.join("var");
        let cache_dir = var_dir.join("cache");
        Self {
            config_file: root.join("config.toml"),
            packages_file: root.join("packages.toml"),
            sources_file: root.join("sources.toml"),
            seen_file: root.join("seen.toml"),
            scripts_file: root.join("scripts.toml"),
            active_dir: root.join("active"),
            bin_dir: root.join("bin"),
            scripts_dir: root.join("scripts"),
            plugins_dir: root.join("plugins"),
            var_dir,
            cache_dir,
            root,
        }
    }

    pub fn resolve() -> Self {
        if let Ok(env_root) = std::env::var("LACIE_HOME")
            && !env_root.trim().is_empty()
        {
            return Self::from_root(PathBuf::from(env_root));
        }
        #[cfg(windows)]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                return Self::from_root(PathBuf::from(local_app_data).join("lacie"));
            }
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                return Self::from_root(PathBuf::from(user_profile).join("AppData").join("Local").join("lacie"));
            }
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self::from_root(PathBuf::from(home).join("lacie"))
    }

    pub fn ensure_dir_permissions(path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            let _ = std::fs::set_permissions(path, perms);
        }
        Ok(())
    }

    pub fn ensure_all_dirs(&self) -> std::io::Result<()> {
        Self::ensure_dir_permissions(&self.root)?;
        Self::ensure_dir_permissions(&self.active_dir)?;
        Self::ensure_dir_permissions(&self.bin_dir)?;
        Self::ensure_dir_permissions(&self.scripts_dir)?;
        Self::ensure_dir_permissions(&self.plugins_dir)?;
        Self::ensure_dir_permissions(&self.var_dir)?;
        Self::ensure_dir_permissions(&self.cache_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_from_root() {
        let p = LaciePaths::from_root(PathBuf::from("/custom/lacie"));
        assert_eq!(p.root, PathBuf::from("/custom/lacie"));
        assert_eq!(p.config_file, PathBuf::from("/custom/lacie/config.toml"));
        assert_eq!(p.active_dir, PathBuf::from("/custom/lacie/active"));
        assert_eq!(p.cache_dir, PathBuf::from("/custom/lacie/var/cache"));
    }
}
