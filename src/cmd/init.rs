use crate::config::paths::LaciePaths;
use crate::config::shell;
use crate::config::{save_config, Config};
use crate::diag::{print_warning, DiagError, DiagWarning};
use crate::locale;
use std::collections::HashSet;
use std::io::{stdin, stdout, IsTerminal, Write};
use std::path::Path;

fn read_line_trimmed() -> String {
    let mut input = String::new();
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut input);
    input.trim().to_string()
}

fn ask_choice(prompt_key: &str, default_choice: &str) -> String {
    let prompt_text = locale::format(prompt_key, &[("default", default_choice)]);
    print!("      {} ", prompt_text);
    let ans = read_line_trimmed();
    if ans.is_empty() {
        default_choice.to_string()
    } else {
        ans
    }
}

fn ask_yes_no(prompt_key: &str, default_yes: bool) -> bool {
    let default_str = if default_yes { "Y/n" } else { "y/N" };
    let prompt_text = locale::t(prompt_key);
    let formatted = if prompt_text.contains("{default}") {
        prompt_text.replace("{default}", default_str)
    } else {
        std::format!("[{}]:", default_str)
    };
    print!("      {} ", formatted);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestionDef {
    pub id: &'static str,
    pub aliases: &'static [&'static str],
    pub config_key: Option<&'static str>,
}

impl QuestionDef {
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        self.id == q || self.aliases.iter().any(|&a| a == q)
    }

    pub fn is_recorded(&self, recorded_keys: &HashSet<String>, paths: &LaciePaths) -> bool {
        if let Some(key) = self.config_key {
            recorded_keys.contains(key)
        } else {
            is_path_recorded(paths)
        }
    }
}

pub static QUESTION_REGISTRY: &[QuestionDef] = &[
    QuestionDef {
        id: "language",
        aliases: &["1", "lang"],
        config_key: Some("language"),
    },
    QuestionDef {
        id: "colors",
        aliases: &["2", "color"],
        config_key: Some("colors"),
    },
    QuestionDef {
        id: "verify_sha256",
        aliases: &["3", "verify", "sha256", "verify-sha256"],
        config_key: Some("verify_sha256"),
    },
    QuestionDef {
        id: "path",
        aliases: &["4"],
        config_key: None,
    },
    QuestionDef {
        id: "auto_switch",
        aliases: &["5", "switch", "auto-switch"],
        config_key: Some("auto_switch"),
    },
];

fn is_path_recorded(paths: &LaciePaths) -> bool {
    if let Ok(val) = std::env::var("PATH")
        && std::env::split_paths(&val).any(|p| p == paths.active_dir)
    {
        return true;
    }
    let shell = shell::detect_shell_name();
    if let Some(file) = shell::get_shell_config_file(&shell)
        && file.exists()
        && let Ok(content) = std::fs::read_to_string(&file)
    {
        let active_str = paths.active_dir.to_string_lossy();
        return content.contains("# >>> lacie >>>") || content.contains(&*active_str);
    }
    false
}

fn read_recorded_keys(config_file: &Path) -> HashSet<String> {
    if !config_file.is_file() {
        return HashSet::new();
    }
    std::fs::read_to_string(config_file)
        .ok()
        .and_then(|c| toml::from_str::<toml::Table>(&c).ok())
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default()
}

fn ask_language(cfg: &mut Config) {
    println!("\n{}", locale::t("cmd_init.q1_title"));
    println!("      {}", locale::t("cmd_init.q1_opt1"));
    println!("      {}", locale::t("cmd_init.q1_opt2"));
    println!("      {}", locale::t("cmd_init.q1_opt3"));
    let def = match cfg.language.as_str() {
        "en" => "2",
        "ru" => "3",
        _ => "1",
    };
    let choice = ask_choice("cmd_init.q1_prompt", def);
    cfg.language = match choice.as_str() {
        "2" => "en".to_string(),
        "3" => "ru".to_string(),
        _ => "auto".to_string(),
    };
    locale::set_language(&cfg.language);
}

fn ask_colors(cfg: &mut Config) {
    println!("\n{}", locale::t("cmd_init.q2_title"));
    println!("      {}", locale::t("cmd_init.q2_opt1"));
    println!("      {}", locale::t("cmd_init.q2_opt2"));
    println!("      {}", locale::t("cmd_init.q2_opt3"));
    let def = match cfg.colors.as_str() {
        "always" => "2",
        "never" => "3",
        _ => "1",
    };
    let choice = ask_choice("cmd_init.q2_prompt", def);
    cfg.colors = match choice.as_str() {
        "2" => "always".to_string(),
        "3" => "never".to_string(),
        _ => "terminal".to_string(),
    };
}

fn ask_verify(cfg: &mut Config) {
    println!("\n{}", locale::t("cmd_init.q3_title"));
    cfg.verify_sha256 = ask_yes_no("cmd_init.q3_prompt", cfg.verify_sha256);
}

fn ask_path(paths: &LaciePaths) -> bool {
    let shell_name = shell::detect_shell_name();
    let shell_file = shell::get_shell_config_file(&shell_name);
    let shell_display = shell_file
        .as_ref()
        .map(|f| f.display().to_string())
        .unwrap_or_else(|| "profile".to_string());
    let title = locale::format(
        "cmd_init.q4_title",
        &[("dir", &paths.active_dir.display().to_string()), ("file", &shell_display)],
    );
    println!("\n{}", title);
    ask_yes_no("cmd_init.q4_prompt", true)
}

fn ask_switch(cfg: &mut Config) {
    println!("\n{}", locale::t("cmd_init.q5_title"));
    println!("      {}", locale::t("cmd_init.q5_note"));
    cfg.auto_switch = ask_yes_no("cmd_init.q5_prompt", cfg.auto_switch);
}

fn select_questions_to_run<'a>(
    question: Option<&str>,
    recfg: bool,
    recorded_keys: &HashSet<String>,
    paths: &LaciePaths,
) -> Result<Vec<&'a QuestionDef>, DiagError> {
    if let Some(target) = question {
        let matched = QUESTION_REGISTRY
            .iter()
            .find(|q| q.matches(target))
            .ok_or_else(|| {
                DiagError::from_code(
                    "E017",
                    &[("expected", "valid question id (1..5, language, colors, verify_sha256, path, auto_switch)")],
                )
            })?;
        return Ok(vec![matched]);
    }
    if recfg {
        return Ok(QUESTION_REGISTRY.iter().collect());
    }
    let filtered = QUESTION_REGISTRY
        .iter()
        .filter(|q| !q.is_recorded(recorded_keys, paths))
        .collect();
    Ok(filtered)
}

fn execute_single_question(
    q: &QuestionDef,
    paths: &LaciePaths,
    cfg: &mut Config,
) -> Option<bool> {
    match q.id {
        "language" => {
            ask_language(cfg);
            None
        }
        "colors" => {
            ask_colors(cfg);
            None
        }
        "verify_sha256" => {
            ask_verify(cfg);
            None
        }
        "path" => Some(ask_path(paths)),
        "auto_switch" => {
            ask_switch(cfg);
            None
        }
        _ => None,
    }
}

fn print_init_summary(paths: &LaciePaths, updated_shell: Option<&Path>) {
    println!("\n{}   {}", locale::t("cmd_init.created"), paths.root.display());
    println!(
        "{}   config.toml  packages.toml  sources.toml  seen.toml  scripts.toml",
        locale::t("cmd_init.created")
    );
    println!("{}   active/  bin/  scripts/  plugins/  var/  var/cache/", locale::t("cmd_init.created"));
    if let Some(sh_file) = updated_shell {
        println!("{}   {}", locale::t("cmd_init.updated"), sh_file.display());
        let cmd = format!("source {}", sh_file.display());
        let msg = locale::format("cmd_init.done", &[("command", &cmd)]);
        println!("\n{}", msg);
    } else {
        println!("\n{}", locale::t("cmd_init.done_simple"));
    }
}

pub fn execute_init(paths: &LaciePaths, question: Option<&str>, recfg: bool) -> Result<(), DiagError> {
    if !stdin().is_terminal() {
        return Err(DiagError::from_code("E002", &[("command", "init-lacie")]));
    }
    let dir_msg = locale::format("cmd_init.lacie_dir", &[("dir", &paths.root.display().to_string())]);
    println!("{}", dir_msg);
    println!("{}", locale::t("cmd_init.lacie_dir_desc"));

    let recorded_keys = read_recorded_keys(&paths.config_file);
    let mut cfg = if paths.config_file.is_file() {
        std::fs::read_to_string(&paths.config_file)
            .ok()
            .and_then(|c| toml::from_str::<Config>(&c).ok())
            .unwrap_or_default()
    } else {
        Config::default()
    };

    let questions = select_questions_to_run(question, recfg, &recorded_keys, paths)?;
    let mut add_path_requested = None;
    for q in questions {
        if let Some(add_path) = execute_single_question(q, paths, &mut cfg) {
            add_path_requested = Some(add_path);
        }
    }

    paths.ensure_all_dirs().map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.packages_file).map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.sources_file).map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.seen_file).map_err(|e| DiagError::new("E015", e.to_string()))?;
    ensure_empty_file_exists(&paths.scripts_file).map_err(|e| DiagError::new("E015", e.to_string()))?;

    let updated_shell = if add_path_requested.unwrap_or(false) {
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

    cfg.init = true;
    save_config(paths, &cfg)?;
    print_init_summary(paths, updated_shell.as_deref());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_matching() {
        let q1 = &QUESTION_REGISTRY[0];
        assert_eq!(q1.id, "language");
        assert!(q1.matches("1"));
        assert!(q1.matches("language"));
        assert!(q1.matches("LANGUAGE"));
        assert!(q1.matches("lang"));
        assert!(!q1.matches("2"));

        let q2 = &QUESTION_REGISTRY[1];
        assert!(q2.matches("2"));
        assert!(q2.matches("colors"));
        assert!(q2.matches("color"));

        let q3 = &QUESTION_REGISTRY[2];
        assert!(q3.matches("3"));
        assert!(q3.matches("verify"));
        assert!(q3.matches("verify_sha256"));
        assert!(q3.matches("verify-sha256"));

        let q4 = &QUESTION_REGISTRY[3];
        assert!(q4.matches("4"));
        assert!(q4.matches("path"));

        let q5 = &QUESTION_REGISTRY[4];
        assert!(q5.matches("5"));
        assert!(q5.matches("switch"));
        assert!(q5.matches("auto_switch"));
        assert!(q5.matches("auto-switch"));
    }

    #[test]
    fn test_select_single_question() {
        let tmp = std::env::temp_dir().join("lacie_test_init_single");
        let paths = LaciePaths::from_root(tmp);
        let recorded = HashSet::new();

        let res = select_questions_to_run(Some("language"), false, &recorded, &paths).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, "language");

        let res_num = select_questions_to_run(Some("3"), false, &recorded, &paths).unwrap();
        assert_eq!(res_num.len(), 1);
        assert_eq!(res_num[0].id, "verify_sha256");

        let err = select_questions_to_run(Some("invalid"), false, &recorded, &paths);
        assert!(err.is_err());
    }

    #[test]
    fn test_select_questions_with_recfg_and_recorded() {
        let tmp = std::env::temp_dir().join("lacie_test_init_recfg");
        let paths = LaciePaths::from_root(tmp);
        let mut recorded = HashSet::new();
        recorded.insert("language".to_string());
        recorded.insert("colors".to_string());

        let res_filtered = select_questions_to_run(None, false, &recorded, &paths).unwrap();
        assert!(!res_filtered.iter().any(|q| q.id == "language"));
        assert!(!res_filtered.iter().any(|q| q.id == "colors"));
        assert!(res_filtered.iter().any(|q| q.id == "verify_sha256"));

        let res_recfg = select_questions_to_run(None, true, &recorded, &paths).unwrap();
        assert_eq!(res_recfg.len(), 5);
    }

    #[test]
    fn test_read_recorded_keys() {
        let tmp = std::env::temp_dir().join("lacie_test_recorded_keys");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let cfg_path = tmp.join("config.toml");

        assert!(read_recorded_keys(&cfg_path).is_empty());

        std::fs::write(&cfg_path, "language = \"ru\"\ncolors = \"never\"\n").unwrap();
        let keys = read_recorded_keys(&cfg_path);
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("language"));
        assert!(keys.contains("colors"));
        assert!(!keys.contains("verify_sha256"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
