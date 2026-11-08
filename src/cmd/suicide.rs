use crate::config::paths::LaciePaths;
use crate::config::shell::{detect_shell_name, get_shell_config_file, remove_active_from_path};
use crate::config::Config;
use crate::diag::DiagError;
use crate::locale;
use crate::state::packages::PackagesDatabase;
use crate::state::sources::SourcesDatabase;
use crate::ui::prompt::prompt_yes_no;

pub fn execute_suicide(paths: &LaciePaths, cfg: &Config, args: &[String]) -> Result<(), DiagError> {
    let no_scripts = args.iter().any(|a| a == "--no-scripts");
    let pkgs_count = PackagesDatabase::load(&paths.packages_file).map(|db| db.packages.len()).unwrap_or(0);
    let sources_count = SourcesDatabase::load(&paths.sources_file).map(|db| db.sources.len()).unwrap_or(0);
    let current_exe = std::env::current_exe().unwrap_or_default();
    let shell_name = detect_shell_name();
    let shell_cfg = get_shell_config_file(&shell_name).map(|p| p.display().to_string()).unwrap_or_else(|| "shell profile".to_string());

    println!("{}\n", locale::t("cmd_suicide.will_be_removed"));
    println!("  {:<10} {} ({} packages, {} sources)", locale::t("cmd_suicide.directory_label"), paths.root.display(), pkgs_count, sources_count);
    println!("  {:<10} {}", locale::t("cmd_suicide.binary_label"), current_exe.display());
    println!("  {:<10} entry in {}", locale::t("cmd_suicide.path_label"), shell_cfg);
    if !no_scripts && pkgs_count > 0 {
        println!("  {:<10} preremove and postremove of {} packages will run first", locale::t("cmd_suicide.scripts_label"), pkgs_count);
    }
    println!();

    let q = locale::t("cmd_suicide.prompt");
    if !prompt_yes_no(&q, false, "suicide")? {
        return Ok(());
    }

    if !no_scripts && pkgs_count > 0 {
        let _ = crate::cmd::remove::execute_remove(paths, cfg, &["ALL".to_string()]);
    }
    let _ = remove_active_from_path();

    if paths.root.is_dir() {
        std::fs::remove_dir_all(&paths.root).map_err(|e| DiagError::new("E016", e.to_string()))?;
    }
    if current_exe.is_file() {
        let _ = std::fs::remove_file(current_exe);
    }
    println!("lacie has been successfully removed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suicide_detection() {
        let tmp = std::env::temp_dir().join("lacie_test_suicide");
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LaciePaths::from_root(tmp);
        paths.ensure_all_dirs().unwrap();
        assert!(paths.root.is_dir());
    }
}
