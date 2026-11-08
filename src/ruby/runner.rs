use crate::diag::DiagError;
use std::path::Path;
use std::process::Command;

pub struct HookContext<'a> {
    pub pkg_id: &'a str,
    pub pkg_version: &'a str,
    pub pkg_version_number: u64,
    pub target: &'a str,
    pub dest_dir: &'a Path,
    pub hook_name: &'a str,
    pub is_last: bool,
}

pub fn ensure_ruby_available() -> Result<(), DiagError> {
    match Command::new("ruby").arg("-v").output() {
        Ok(out) if out.status.success() => Ok(()),
        _ => Err(DiagError::from_code("E036", &[])),
    }
}

pub fn execute_ruby_script(
    script_path: &Path,
    work_dir: &Path,
    ctx: &HookContext,
) -> Result<(), DiagError> {
    ensure_ruby_available()?;
    let status = Command::new("ruby")
        .arg(script_path)
        .current_dir(work_dir)
        .env("LACIE_PKG_ID", ctx.pkg_id)
        .env("LACIE_PKG_VERSION", ctx.pkg_version)
        .env("LACIE_PKG_VERSION_NUMBER", ctx.pkg_version_number.to_string())
        .env("LACIE_TARGET", ctx.target)
        .env("LACIE_DEST_DIR", ctx.dest_dir)
        .env("LACIE_HOOK", ctx.hook_name)
        .env("LACIE_LAST", if ctx.is_last { "true" } else { "false" })
        .status()
        .map_err(|e| DiagError::new("E033", e.to_string()))?;

    if !status.success() {
        let code = match ctx.hook_name {
            "check_deps" => "E026",
            "prebuild" => "E022",
            "postbuild" => "E023",
            "preremove" => "E025",
            _ => "E022",
        };
        let reason = std::format!("exit status {}", status);
        return Err(DiagError::from_code(code, &[("package", ctx.pkg_id), ("reason", &reason)]));
    }
    Ok(())
}
