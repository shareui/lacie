use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Terminal,
    Always,
    Never,
}

impl std::str::FromStr for ColorMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mode = match s.to_lowercase().as_str() {
            "always" => Self::Always,
            "never" => Self::Never,
            _ => Self::Terminal,
        };
        Ok(mode)
    }
}

pub struct Styler {
    enabled: bool,
}

impl Styler {
    pub fn new(mode: ColorMode) -> Self {
        let is_term = std::io::stdout().is_terminal();
        let enabled = match mode {
            ColorMode::Never => false,
            ColorMode::Always => is_term,
            ColorMode::Terminal => {
                if !is_term || std::env::var("NO_COLOR").is_ok() {
                    false
                } else {
                    std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
                }
            }
        };
        Self { enabled }
    }

    pub fn is_colored(&self) -> bool {
        self.enabled
    }

    pub fn cyan(&self, text: &str) -> String {
        self.apply("\x1b[36m", text)
    }

    pub fn bold_cyan(&self, text: &str) -> String {
        self.apply("\x1b[1;36m", text)
    }

    pub fn bold_white(&self, text: &str) -> String {
        self.apply("\x1b[1;37m", text)
    }

    pub fn green(&self, text: &str) -> String {
        self.apply("\x1b[32m", text)
    }

    pub fn bold_green(&self, text: &str) -> String {
        self.apply("\x1b[1;32m", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.apply("\x1b[2m", text)
    }

    pub fn underline(&self, text: &str) -> String {
        self.apply("\x1b[4m", text)
    }

    fn apply(&self, code: &str, text: &str) -> String {
        if self.enabled {
            std::format!("{}{}\x1b[0m", code, text)
        } else {
            text.to_string()
        }
    }
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_esc = true;
        } else if in_esc {
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn visible_len(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        let colored = "\x1b[1;36m::\x1b[0m Resolving package \x1b[1;37mapp\x1b[0m";
        assert_eq!(strip_ansi(colored), ":: Resolving package app");
        assert_eq!(visible_len(colored), 24);
    }
}
