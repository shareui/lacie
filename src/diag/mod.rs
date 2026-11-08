pub mod explain;
pub mod registry;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagError {
    pub code: String,
    pub message: String,
}

impl DiagError {
    pub fn new(code: &str, message: String) -> Self {
        Self {
            code: code.to_string(),
            message,
        }
    }

    pub fn from_code(code: &str, params: &[(&str, &str)]) -> Self {
        let raw_tmpl = registry::find_diag(code)
            .map(|item| item.summary_template)
            .unwrap_or(code);
        let mut msg = raw_tmpl.to_string();
        for (k, v) in params {
            msg = msg.replace(&std::format!("{{{}}}", k), v);
        }
        Self::new(code, msg)
    }

    pub fn numeric_code(&self) -> &str {
        if self.code.starts_with('E') || self.code.starts_with('W') {
            &self.code[1..]
        } else {
            &self.code
        }
    }
}

impl fmt::Display for DiagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error [{}]: {}", self.numeric_code(), self.message)
    }
}

impl std::error::Error for DiagError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagWarning {
    pub code: String,
    pub message: String,
}

impl DiagWarning {
    pub fn new(code: &str, message: String) -> Self {
        Self {
            code: code.to_string(),
            message,
        }
    }

    pub fn from_code(code: &str, params: &[(&str, &str)]) -> Self {
        let raw_tmpl = registry::find_diag(code)
            .map(|item| item.summary_template)
            .unwrap_or(code);
        let mut msg = raw_tmpl.to_string();
        for (k, v) in params {
            msg = msg.replace(&std::format!("{{{}}}", k), v);
        }
        Self::new(code, msg)
    }

    pub fn numeric_code(&self) -> &str {
        if self.code.starts_with('W') || self.code.starts_with('E') {
            &self.code[1..]
        } else {
            &self.code
        }
    }
}

impl fmt::Display for DiagWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Warning [{}]: {}", self.numeric_code(), self.message)
    }
}

pub fn print_error(err: &DiagError) {
    eprintln!("{}", err);
}

pub fn print_warning(warn: &DiagWarning) {
    eprintln!("{}", warn);
}

pub fn exit_error(err: &DiagError) -> ! {
    print_error(err);
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let err = DiagError::from_code("E001", &[]);
        assert_eq!(
            err.to_string(),
            "Error [001]: Configuration not initialized, use `lacie init-lacie`"
        );
    }

    #[test]
    fn test_warning_formatting() {
        let warn = DiagWarning::from_code("W001", &[("path", "/test/lacie/active")]);
        assert_eq!(
            warn.to_string(),
            "Warning [001]: Could not add /test/lacie/active to PATH, lacie will not work until you add it manually, use `lacie explain W001`"
        );
    }
}
