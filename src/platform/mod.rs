pub mod fs;
pub mod target;

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsType {
    Linux,
    Windows,
    MacOS,
    FreeBSD,
    OpenBSD,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchType {
    X86_64,
    Aarch64,
    Other,
}

pub fn current_os() -> OsType {
    match std::env::consts::OS {
        "linux" => OsType::Linux,
        "windows" => OsType::Windows,
        "macos" => OsType::MacOS,
        "freebsd" => OsType::FreeBSD,
        "openbsd" => OsType::OpenBSD,
        _ => OsType::Other,
    }
}

pub fn current_arch() -> ArchType {
    match std::env::consts::ARCH {
        "x86_64" => ArchType::X86_64,
        "aarch64" => ArchType::Aarch64,
        _ => ArchType::Other,
    }
}

pub fn has_musl_linker(arch_str: &str) -> bool {
    let candidate_names = [
        std::format!("ld-musl-{}.so.1", arch_str),
        std::format!("ld-musl-{}.so", arch_str),
    ];
    let candidate_dirs = ["/lib", "/usr/lib", "/lib64", "/usr/local/lib"];
    for dir in candidate_dirs {
        for name in &candidate_names {
            if Path::new(dir).join(name).exists() {
                return true;
            }
        }
    }
    false
}
