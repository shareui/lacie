use crate::ui::color::Styler;

pub fn generate_unified_diff(old_text: &str, new_text: &str, styler: &Styler) -> String {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let mut out = String::new();

    let mut i = 0;
    let mut j = 0;
    while i < old_lines.len() || j < new_lines.len() {
        if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
            out.push_str(&std::format!("  {}\n", old_lines[i]));
            i += 1;
            j += 1;
        } else if i < old_lines.len() && (j >= new_lines.len() || !new_lines.contains(&old_lines[i])) {
            let line = std::format!("- {}", old_lines[i]);
            out.push_str(&std::format!("{}\n", styler.dim(&line)));
            i += 1;
        } else if j < new_lines.len() {
            let line = std::format!("+ {}", new_lines[j]);
            out.push_str(&std::format!("{}\n", styler.green(&line)));
            j += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::color::ColorMode;

    #[test]
    fn test_diff_rendering() {
        let old = "version = \"0.1.0\"\nauthor = \"shareui\"\n";
        let new = "version = \"0.2.0\"\nauthor = \"shareui\"\n";
        let styler = Styler::new(ColorMode::Never);
        let diff = generate_unified_diff(old, new, &styler);
        assert!(diff.contains("- version = \"0.1.0\""));
        assert!(diff.contains("+ version = \"0.2.0\""));
        assert!(diff.contains("  author = \"shareui\""));
    }
}
