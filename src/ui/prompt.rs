use crate::diag::DiagError;
use std::io::{stdin, stdout, IsTerminal, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptResult {
    Yes,
    No,
    View,
}

pub fn prompt_yes_no(question: &str, default_yes: bool, cmd_name: &str) -> Result<bool, DiagError> {
    if !stdin().is_terminal() {
        return Err(DiagError::from_code("E002", &[("command", cmd_name)]));
    }
    let choice_hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{} {} ", question, choice_hint);
    let _ = stdout().flush();

    let mut line = String::new();
    stdin().read_line(&mut line)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let trimmed = line.trim().to_lowercase();
    if trimmed.is_empty() {
        Ok(default_yes)
    } else {
        Ok(trimmed == "y" || trimmed == "yes")
    }
}

pub fn prompt_yes_no_view(question: &str, default_yes: bool, cmd_name: &str) -> Result<PromptResult, DiagError> {
    if !stdin().is_terminal() {
        return Err(DiagError::from_code("E002", &[("command", cmd_name)]));
    }
    let choice_hint = if default_yes { "[Y/n/v]" } else { "[y/N/v]" };
    print!("{} {} ", question, choice_hint);
    let _ = stdout().flush();

    let mut line = String::new();
    stdin().read_line(&mut line)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let trimmed = line.trim().to_lowercase();
    if trimmed.is_empty() {
        Ok(if default_yes { PromptResult::Yes } else { PromptResult::No })
    } else if trimmed == "v" || trimmed == "view" {
        Ok(PromptResult::View)
    } else if trimmed == "y" || trimmed == "yes" {
        Ok(PromptResult::Yes)
    } else {
        Ok(PromptResult::No)
    }
}
