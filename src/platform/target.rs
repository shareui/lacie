use crate::platform::{current_arch, current_os, has_musl_linker, ArchType, OsType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemTarget {
    pub os: String,
    pub abi: Option<String>,
    pub arch: String,
}

impl SystemTarget {
    pub fn as_str(&self) -> String {
        match &self.abi {
            Some(abi) => std::format!("{}-{}-{}", self.os, abi, self.arch),
            None => std::format!("{}-{}", self.os, self.arch),
        }
    }
}

pub fn detect_current_target(windows_priority: &str) -> SystemTarget {
    let arch_str = match current_arch() {
        ArchType::X86_64 => "x86_64",
        ArchType::Aarch64 => "aarch64",
        ArchType::Other => std::env::consts::ARCH,
    };
    match current_os() {
        OsType::Linux => SystemTarget {
            os: "linux".to_string(),
            abi: Some("gnu".to_string()),
            arch: arch_str.to_string(),
        },
        OsType::Windows => {
            let abi = if windows_priority.to_lowercase() == "mingw" {
                "mingw"
            } else {
                "msvc"
            };
            SystemTarget {
                os: "windows".to_string(),
                abi: Some(abi.to_string()),
                arch: arch_str.to_string(),
            }
        }
        OsType::MacOS => SystemTarget {
            os: "macos".to_string(),
            abi: None,
            arch: arch_str.to_string(),
        },
        OsType::FreeBSD => SystemTarget {
            os: "freebsd".to_string(),
            abi: None,
            arch: arch_str.to_string(),
        },
        OsType::OpenBSD => SystemTarget {
            os: "openbsd".to_string(),
            abi: None,
            arch: arch_str.to_string(),
        },
        OsType::Other => SystemTarget {
            os: std::env::consts::OS.to_string(),
            abi: None,
            arch: arch_str.to_string(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetMatchResult {
    Exact(String),
    Fallback {
        target: String,
        preferred: String,
    },
    NotFound,
}

pub fn match_target(
    available: &[(&str, bool)],
    current: &SystemTarget,
    windows_priority: &str,
) -> TargetMatchResult {
    let exact_str = current.as_str();
    if available.iter().any(|(t, _)| *t == exact_str) {
        return TargetMatchResult::Exact(exact_str);
    }
    if current.os == "linux" && current.abi.as_deref() == Some("gnu") {
        let musl_target = std::format!("linux-musl-{}", current.arch);
        if let Some((_, is_static)) = available.iter().find(|(t, _)| *t == musl_target)
            && (*is_static || has_musl_linker(&current.arch))
        {
            return TargetMatchResult::Fallback {
                target: musl_target,
                preferred: exact_str,
            };
        }
    }
    if current.os == "windows" {
        let alt_abi = if windows_priority == "mingw" { "msvc" } else { "mingw" };
        let alt_target = std::format!("windows-{}-{}", alt_abi, current.arch);
        if available.iter().any(|(t, _)| *t == alt_target) {
            return TargetMatchResult::Fallback {
                target: alt_target,
                preferred: exact_str,
            };
        }
    }
    TargetMatchResult::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let current = SystemTarget {
            os: "linux".to_string(),
            abi: Some("gnu".to_string()),
            arch: "x86_64".to_string(),
        };
        let available = [("linux-gnu-x86_64", false), ("linux-musl-x86_64", true)];
        let res = match_target(&available, &current, "msvc");
        assert_eq!(res, TargetMatchResult::Exact("linux-gnu-x86_64".to_string()));
    }

    #[test]
    fn test_linux_musl_fallback() {
        let current = SystemTarget {
            os: "linux".to_string(),
            abi: Some("gnu".to_string()),
            arch: "x86_64".to_string(),
        };
        let available = [("linux-musl-x86_64", true)];
        let res = match_target(&available, &current, "msvc");
        assert_eq!(
            res,
            TargetMatchResult::Fallback {
                target: "linux-musl-x86_64".to_string(),
                preferred: "linux-gnu-x86_64".to_string(),
            }
        );
    }
}
