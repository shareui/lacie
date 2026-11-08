use crate::diag::registry::{find_diag, DiagItem};

fn format_solutions(item: &DiagItem) -> String {
    let mut out = String::new();
    out.push_str("Solutions:\n");
    for (i, solution) in item.solutions.iter().enumerate() {
        out.push_str(&std::format!("  {}. {}\n", i + 1, solution));
    }
    out
}

fn format_w001_details() -> String {
    let mut out = String::new();
    out.push_str("How to fix:\n");
    out.push_str("  Add the directory to PATH in the startup file of your shell,\n");
    out.push_str("  then open a new terminal. Examples for common shells:\n\n");
    out.push_str("  bash      ~/.bashrc                           export PATH=\"$HOME/lacie/active:$PATH\"\n");
    out.push_str("  zsh       ~/.zshrc                            export PATH=\"$HOME/lacie/active:$PATH\"\n");
    out.push_str("  fish      ~/.config/fish/conf.d/lacie.fish    fish_add_path $HOME/lacie/active\n");
    out.push_str("  sh, ksh   ~/.profile                          export PATH=\"$HOME/lacie/active:$PATH\"\n");
    out.push_str("  csh       ~/.cshrc                            set path = ($HOME/lacie/active $path)\n");
    out.push_str("  Windows   add %LOCALAPPDATA%\\lacie\\active to the user Path variable\n");
    out
}

pub fn render_explain(code: &str) -> Option<String> {
    let upper = code.trim().to_uppercase();
    let diag = find_diag(&upper)
        .or_else(|| {
            if upper.chars().all(|c| c.is_ascii_digit()) {
                find_diag(&std::format!("E{:0>3}", upper))
                    .or_else(|| find_diag(&std::format!("W{:0>3}", upper)))
            } else {
                None
            }
        })?;

    let mut out = String::new();
    out.push_str(&std::format!("{}: {}\n\n", diag.code, diag.title));
    out.push_str(diag.description);
    out.push_str("\n\n");
    if diag.code == "W001" {
        out.push_str(&format_w001_details());
    } else {
        out.push_str(&format_solutions(diag));
    }
    Some(out)
}

pub fn explain_command(code: &str) -> Result<(), String> {
    match render_explain(code) {
        Some(text) => {
            print!("{}", text);
            Ok(())
        }
        None => Err(std::format!("Unknown diagnostic code `{}`", code)),
    }
}
