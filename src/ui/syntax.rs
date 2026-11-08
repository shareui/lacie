use crate::ui::color::Styler;

pub fn print_manifest_view(
    content: &str,
    storage: &str,
    release: &str,
    chosen_targets: &[&str],
    selected_pkgs: &[String],
    styler: &Styler,
) {
    println!("lacie.toml  {}  {}\n", storage, release);
    let target_label = chosen_targets.first().copied().unwrap_or("current target");
    if styler.is_colored() {
        println!("Dimmed lines do not apply to this installation ({})\n", target_label);
    } else {
        println!("Lines marked with ~ do not apply to this installation ({})\n", target_label);
    }

    let mut is_dimmed_block = false;
    let mut current_package = String::new();

    for (i, line) in content.lines().enumerate() {
        let line_no = i + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            is_dimmed_block = determine_header_dimmed(
                trimmed,
                chosen_targets,
                selected_pkgs,
                &mut current_package,
            );
        }

        render_syntax_line(line, line_no, is_dimmed_block, styler);
    }
}

pub fn determine_header_dimmed(
    trimmed: &str,
    chosen_targets: &[&str],
    selected_pkgs: &[String],
    current_package: &mut String,
) -> bool {
    if trimmed.starts_with("[packages.") {
        if trimmed.contains(".binaries.") {
            for t in chosen_targets {
                if trimmed.contains(&std::format!(".binaries.\"{}\"", t))
                    || trimmed.contains(&std::format!(".binaries.{}", t))
                {
                    return !selected_pkgs.is_empty() && !selected_pkgs.contains(current_package);
                }
            }
            true
        } else {
            let pkg_name = trimmed.trim_start_matches("[packages.").trim_end_matches(']').trim_matches('"');
            *current_package = pkg_name.to_string();
            !selected_pkgs.is_empty() && !selected_pkgs.contains(current_package)
        }
    } else {
        false
    }
}

fn render_syntax_line(line: &str, line_no: usize, is_dimmed: bool, styler: &Styler) {
    if styler.is_colored() {
        let line_num_str = styler.dim(&std::format!("{:3}  ", line_no));
        if is_dimmed {
            println!("{}{}", line_num_str, styler.dim(line));
        } else {
            let highlighted = highlight_toml_line(line, styler);
            println!("{}{}", line_num_str, highlighted);
        }
    } else if is_dimmed && !line.trim().is_empty() {
        println!("~{:3}  {}", line_no, line);
    } else {
        println!(" {:3}  {}", line_no, line);
    }
}

pub fn highlight_toml(content: &str, _current_target: &str, styler: &Styler) -> String {
    let mut out = String::new();
    for line in content.lines() {
        out.push_str(&highlight_toml_line(line, styler));
        out.push('\n');
    }
    out
}

pub fn highlight_toml_line(line: &str, styler: &Styler) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return styler.dim(line);
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return styler.bold_white(line);
    }
    if let Some((key, val)) = line.split_once('=') {
        let key_trimmed = key.trim();
        let val_trimmed = val.trim();
        let val_highlighted = if (val_trimmed.starts_with('"') && val_trimmed.ends_with('"'))
            || (val_trimmed.starts_with('\'') && val_trimmed.ends_with('\''))
        {
            styler.green(val_trimmed)
        } else if val_trimmed == "true" || val_trimmed == "false" || val_trimmed.parse::<i64>().is_ok() {
            styler.cyan(val_trimmed)
        } else {
            val_trimmed.to_string()
        };
        let leading_ws_len = key.len() - key.trim_start().len();
        let indent = &key[..leading_ws_len];
        std::format!("{}{} = {}", indent, styler.bold_white(key_trimmed), val_highlighted)
    } else {
        line.to_string()
    }
}

pub fn highlight_ruby(content: &str, styler: &Styler) -> String {
    let mut out = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            out.push_str(&std::format!("{}\n", styler.dim(line)));
        } else {
            out.push_str(&std::format!("{}\n", line));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::color::ColorMode;

    #[test]
    fn test_highlight_toml_line_no_duplication() {
        let styler = Styler::new(ColorMode::Never);
        let line = "  name = \"nyano\"";
        let out = highlight_toml_line(line, &styler);
        assert_eq!(out, "  name = \"nyano\"");
        assert!(!out.contains("nname"));
    }
}
