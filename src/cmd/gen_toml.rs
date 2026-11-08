use crate::diag::DiagError;
use crate::locale;
use crate::ui::prompt::prompt_yes_no;
use std::path::Path;

const TEMPLATE_TOML: &str = r#"# lacie.toml: workspace description.
# Values in [workspace] are shared by all packages
# and can be overridden inside a package.

[workspace]
author = ""
version = "0.1.0"
version_number = 1
source_code = ""
links = []

[packages."com.example.app"]
name = "app"
# apps = [{ id = "org.other.tool", from = "github:owner/tool", min_number = 1 }]
# libs = [{ id = "org.other.lib", from = "github:owner/lib", dest = "lib" }]
# check_deps = "check_deps.rb"
# prebuild = "prebuild.rb"
# postbuild = "postbuild.rb"

[packages."com.example.app".binaries."linux-gnu-x86_64"]
file = "app-linux-gnu-x86_64"
# Instead of file, a binary can come from direct links or from another repository:
# links = ["https://example.com/app-linux-gnu-x86_64"]
# source = "github:owner/repo#com.example.app"
# sha256 = "<64 hexadecimal characters>"

[packages."com.example.app".binaries."windows-msvc-x86_64"]
file = "app-windows-msvc-x86_64.exe"
# sha256 = "<64 hexadecimal characters>"

[packages."com.example.app".binaries."macos-aarch64"]
file = "app-macos-aarch64"
# sha256 = "<64 hexadecimal characters>"
"#;

pub fn execute_gen_toml() -> Result<(), DiagError> {
    let dest = Path::new("lacie.toml");
    if dest.exists() {
        let q = locale::t("cmd_gen_toml.overwrite_prompt");
        if !prompt_yes_no(&q, false, "gen-toml")? {
            return Ok(());
        }
    }
    std::fs::write(dest, TEMPLATE_TOML).map_err(|e| DiagError::new("E015", e.to_string()))?;
    println!("{}", locale::t("cmd_gen_toml.created"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_validity() {
        assert!(TEMPLATE_TOML.contains("[workspace]"));
        assert!(TEMPLATE_TOML.contains("[packages.\"com.example.app\"]"));
    }
}
