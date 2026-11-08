use crate::config::paths::LaciePaths;
use crate::config::shell;
use crate::config::{save_config, Config};
use crate::diag::{print_warning, DiagError, DiagWarning};
use crate::locale;
use std::io::{stdin, stdout, IsTerminal, Write};
use std::path::Path;

fn read_line_trimmed() -> String {
    let mut input = String::new();
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut input);
    input.trim().to_string()
}

fn ask_choice(default_choice: &str) -> String {
    print!("      Select [{}]: ", default_choice);
    let ans = read_line_trimmed();
    if ans.is_empty() {
        default_choice.to_string()
    } else {
        ans
    }
}

fn ask_yes_no(default_yes: bool) -> bool {
    let default_str = if default_yes { "Y/n" } else { "y/N" };
    print!("      [{}]: ", default_str);
    let ans = read_line_trimmed().to_lowercase();
    if ans.is_empty() {
        default_yes
    } else {
        ans == "y" || ans == "yes"
    }
}

fn ensure_empty_file_exists(path: &Path) -> std::io::Result<bool> {
    if !path.exists() {
        std::fs::write(path, "")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

fn run_questions(paths: &LaciePaths, existing_cfg: Option<&Config>) -> Result<(Config, bool), DiagError> {
    println!("\n[1/5] Language");
    println!("      1. Auto (system, falls back to English)");
    println!("      2. English");
    println!("      3. Russian");
    let lang_choice = ask_choice("1");
    let language = match lang_choice.as_str() {
        "2" => "en",
        "3" => "ru",
        _ => "auto",
    };
    locale::set_language(language);

    println!("\n[2/5] Colors");
    println!("      1. Terminal settings");
    println!("      2. Always lacie colors");
    println!("      3. No colors");
    let color_choice = ask_choice("1");
    let colors = match color_choice.as_str() {
        "2" => "always",
        "3" => "never",
        _ => "terminal",
    };

    println!("\n[3/5] Verify sha256 of packages (when a package provides a hash)?");
    let verify = ask_yes_no(existing_cfg.map(|c| c.verify_sha256).unwrap_or(true));

    let shell_name = shell::detect_shell_name();
    let shell_file = shell::get_shell_config_file(&shell_name);
    let shell_display = shell_file.as_ref().map(|f| f.display().to_string()).unwrap_or_else(|| "profile".to_string());
    println!("\n[4/5] Add {} to PATH via {}?", paths.active_dir.display(), shell_display);
    let add_path = ask_yes_no(true);

    println!("\n[5/5] Switch to the new version when a new package version is installed?");
    println!("      (no: the active link stays on the old version)");
    let auto_switch = ask_yes_no(existing_cfg.map(|c| c.auto_switch).unwrap_or(true));

    let new_config = Config {
        init: true,
        language: language.to_string(),
        colors: colors.to_string(),
        verify_sha256: verify,
        auto_switch,
        confirm: existing_cfg.map(|c| c.confirm).unwrap_or(true),
        always_show: existing_cfg.map(|c| c.always_show).unwrap_or(false),
        cache_ttl: existing_cfg.map(|c| c.cache_ttl).unwrap_or(10),
        default_source: existing_cfg.map(|c| c.default_source.clone()).unwrap_or_else(|| "github".to_string()),
        windows_priority: existing_cfg.map(|c| c.windows_priority.clone()).unwrap_or_else(|| "msvc".to_string()),
    };
    Ok((new_config, add_path))
}

pub fn execute_init(paths: &LaciePaths) -> Result<(), DiagError> {
    if !stdin().is_terminal() {
        return Err(DiagError::from_code("E002", &[("command", "init-lacie")]));
    }
    println!("lacie directory: {}", paths.root.display());
    println!("lacie works only in this directory, all its files are stored here.");

    let existing_cfg = if paths.config_file.is_file() {
        crate::config::load_config(paths).ok()
    } else {
        None
    };

    if existing_cfg.is_none() {
        paths.ensure_all_dirs().map_err(|e| DiagError::new("E015", e.to_string()))?;
        let interim = Config { init: false, ..Default::default() };
        save_config(paths, &interim)?;
    }

    let (final_config, add_to_path) = run_questions(paths, existing_cfg.as_ref())?;

    paths.ensure_all_dirs().map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.packages_file).map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.sources_file).map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.seen_file).map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.scripts_file).map_err(|e| DiagError::new("E015", e.to_string()))?;

    let updated_shell = if add_to_path {
        match shell::add_active_to_path(&paths.active_dir) {
            Ok(file) => Some(file),
            Err(_) => {
                print_warning(&DiagWarning::from_code("W001", &[("path", &paths.active_dir.display().to_string())]));
                None
            }
        }
    } else {
        None
    };

    save_config(paths, &final_config)?;

    println!("\nCreated   {}", paths.root.display());
    println!("Created   config.toml  packages.toml  sources.toml  seen.toml  scripts.toml");
    println!("Created   active/  bin/  scripts/  plugins/  var/  var/cache/");
    if let Some(ref sh_file) = updated_shell {
        println!("Updated   {}", sh_file.display());
        println!("\nDone. Restart your shell or run `source {}`.", sh_file.display());
    } else {
        println!("\nDone.");
    }
    Ok(())
}
