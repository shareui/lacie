use std::path::{Path, PathBuf};

const START_MARKER: &str = "# >>> lacie >>>";
const END_MARKER: &str = "# <<< lacie <<<";

pub fn detect_shell_name() -> String {
    if let Ok(shell) = std::env::var("SHELL")
        && let Some(name) = Path::new(&shell).file_name()
    {
        return name.to_string_lossy().to_string();
    }
    if cfg!(windows) {
        "powershell".to_string()
    } else {
        "bash".to_string()
    }
}

pub fn get_shell_config_file(shell: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    match shell {
        "bash" => {
            #[cfg(target_os = "macos")]
            return Some(home.join(".bash_profile"));
            #[cfg(not(target_os = "macos"))]
            return Some(home.join(".bashrc"));
        }
        "zsh" => {
            if let Ok(zdotdir) = std::env::var("ZDOTDIR")
                && !zdotdir.trim().is_empty()
            {
                return Some(PathBuf::from(zdotdir).join(".zshrc"));
            }
            Some(home.join(".zshrc"))
        }
        "fish" => Some(home.join(".config").join("fish").join("conf.d").join("lacie.fish")),
        "csh" | "tcsh" => Some(home.join(".cshrc")),
        _ => Some(home.join(".profile")),
    }
}

fn build_export_line(active_dir: &Path, shell: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let path_str = active_dir.to_string_lossy();
    let display_path = if !home.is_empty() && path_str.starts_with(&home) {
        path_str.replacen(&home, "$HOME", 1)
    } else {
        path_str.to_string()
    };
    match shell {
        "fish" => std::format!("fish_add_path {}\n", display_path),
        "csh" | "tcsh" => std::format!("set path = ({} $path)\n", display_path),
        _ => std::format!("export PATH=\"{}:$PATH\"\n", display_path),
    }
}

fn write_fish_config(file: &Path, line: &str) -> std::io::Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, line)
}

fn update_profile_file(file: &Path, line: &str) -> std::io::Result<()> {
    let content = if file.exists() {
        std::fs::read_to_string(file)?
    } else {
        String::new()
    };
    if content.contains(START_MARKER) || content.contains(line.trim()) {
        return Ok(());
    }
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut new_content = content;
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(START_MARKER);
    new_content.push('\n');
    new_content.push_str(line);
    new_content.push_str(END_MARKER);
    new_content.push('\n');
    std::fs::write(file, new_content)
}

pub fn add_active_to_path(active_dir: &Path) -> Result<PathBuf, String> {
    let shell = detect_shell_name();
    let file = get_shell_config_file(&shell)
        .ok_or_else(|| "Could not resolve shell configuration file path".to_string())?;
    let line = build_export_line(active_dir, &shell);
    let res = if shell == "fish" {
        write_fish_config(&file, &line)
    } else {
        update_profile_file(&file, &line)
    };
    res.map(|_| file).map_err(|e| e.to_string())
}

pub fn remove_active_from_path() -> Result<(), String> {
    let shell = detect_shell_name();
    let file = match get_shell_config_file(&shell) {
        Some(f) => f,
        None => return Ok(()),
    };
    if shell == "fish" {
        if file.exists() {
            let _ = std::fs::remove_file(&file);
        }
        return Ok(());
    }
    if !file.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    if let Some(start_idx) = content.find(START_MARKER)
        && let Some(end_idx) = content.find(END_MARKER)
    {
        let end_line_idx = content[end_idx..].find('\n').map(|i| end_idx + i + 1).unwrap_or(content.len());
        let mut new_content = content[..start_idx].to_string();
        new_content.push_str(&content[end_line_idx..]);
        std::fs::write(&file, new_content).map_err(|e| e.to_string())?;
    }
    Ok(())
}
