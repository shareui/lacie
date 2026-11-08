use crate::diag::DiagError;
use crate::ui::color::Styler;
use crate::ui::syntax::{determine_header_dimmed, highlight_toml_line};
use std::io::{stdin, stdout, IsTerminal, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerAction {
    Approved,
    Denied,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerFileType {
    Toml,
    Ruby,
}

#[derive(Debug, Clone)]
pub struct ViewerFile {
    pub title: String,
    pub filename: String,
    pub content: String,
    pub file_type: ViewerFileType,
    pub sha256: String,
    pub chosen_targets: Vec<String>,
    pub selected_pkgs: Vec<String>,
}

pub struct InteractiveViewer<'a> {
    files: Vec<ViewerFile>,
    styler: &'a Styler,
    allow_decision: bool,
}

impl<'a> InteractiveViewer<'a> {
    pub fn new(files: Vec<ViewerFile>, styler: &'a Styler, allow_decision: bool) -> Self {
        Self {
            files,
            styler,
            allow_decision,
        }
    }

    pub fn run(&self) -> Result<ViewerAction, DiagError> {
        if self.files.is_empty() {
            return Ok(ViewerAction::Closed);
        }
        if !stdin().is_terminal() {
            for f in &self.files {
                self.render_file_content(f);
            }
            return Ok(if self.allow_decision { ViewerAction::Approved } else { ViewerAction::Closed });
        }
        let mut idx = 0;
        loop {
            let f = &self.files[idx];
            self.render_header(idx, f);
            self.render_file_content(f);
            self.render_footer();
            let action = self.prompt_user(idx)?;
            match action {
                Some(ViewerAction::Approved) => return Ok(ViewerAction::Approved),
                Some(ViewerAction::Denied) => return Ok(ViewerAction::Denied),
                Some(ViewerAction::Closed) => return Ok(ViewerAction::Closed),
                None => {}
            }
            if let Some(next_idx) = self.handle_navigation(idx, &action) {
                idx = next_idx;
            }
        }
    }

    fn render_header(&self, idx: usize, f: &ViewerFile) {
        let total = self.files.len();
        let bar = "─".repeat(20);
        let title_str = std::format!("File [{}/{}]: {} ({})", idx + 1, total, f.filename, f.title);
        println!("\n{} {} {}", self.styler.dim(&bar), self.styler.bold_white(&title_str), self.styler.dim(&bar));
        println!("SHA256: {}\n", self.styler.dim(&f.sha256));
    }

    fn render_footer(&self) {
        let line = "─".repeat(70);
        println!("{}", self.styler.dim(&line));
    }

    fn render_file_content(&self, f: &ViewerFile) {
        match f.file_type {
            ViewerFileType::Toml => self.render_toml_content(f),
            ViewerFileType::Ruby => self.render_ruby_content(f),
        }
    }

    fn render_toml_content(&self, f: &ViewerFile) {
        let mut is_dimmed_block = false;
        let mut current_package = String::new();
        let targets: Vec<&str> = f.chosen_targets.iter().map(|s| s.as_str()).collect();
        for (i, line) in f.content.lines().enumerate() {
            let line_no = i + 1;
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                is_dimmed_block = determine_header_dimmed(
                    trimmed,
                    &targets,
                    &f.selected_pkgs,
                    &mut current_package,
                );
            }
            self.print_line(line, line_no, is_dimmed_block);
        }
    }

    fn render_ruby_content(&self, f: &ViewerFile) {
        for (i, line) in f.content.lines().enumerate() {
            let line_no = i + 1;
            let num_str = self.styler.dim(&std::format!("{:3}  ", line_no));
            if line.trim().starts_with('#') {
                println!("{}{}", num_str, self.styler.dim(line));
            } else {
                println!("{}{}", num_str, line);
            }
        }
    }

    fn print_line(&self, line: &str, line_no: usize, is_dimmed: bool) {
        if self.styler.is_colored() {
            let num_str = self.styler.dim(&std::format!("{:3}  ", line_no));
            if is_dimmed {
                println!("{}{}", num_str, self.styler.dim(line));
            } else {
                println!("{}{}", num_str, highlight_toml_line(line, self.styler));
            }
        } else if is_dimmed && !line.trim().is_empty() {
            println!("~{:3}  {}", line_no, line);
        } else {
            println!(" {:3}  {}", line_no, line);
        }
    }

    fn prompt_user(&self, idx: usize) -> Result<Option<ViewerAction>, DiagError> {
        let controls = self.build_controls_hint(idx);
        print!("{} ", controls);
        let _ = stdout().flush();
        let mut input = String::new();
        stdin().read_line(&mut input)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        let cmd = input.trim().to_lowercase();
        match cmd.as_str() {
            "a" | "y" | "approve" if self.allow_decision => Ok(Some(ViewerAction::Approved)),
            "d" | "deny" if self.allow_decision => Ok(Some(ViewerAction::Denied)),
            "q" | "quit" => Ok(Some(if self.allow_decision { ViewerAction::Denied } else { ViewerAction::Closed })),
            "" if !self.allow_decision => Ok(Some(ViewerAction::Closed)),
            "n" | "next" if idx + 1 < self.files.len() => Ok(None),
            "p" | "prev" if idx > 0 => Ok(None),
            _ => Ok(None),
        }
    }

    fn build_controls_hint(&self, idx: usize) -> String {
        let mut parts = Vec::new();
        if self.files.len() > 1 {
            if idx + 1 < self.files.len() {
                parts.push(self.styler.cyan("[n]ext file"));
            }
            if idx > 0 {
                parts.push(self.styler.cyan("[p]rev file"));
            }
        }
        if self.allow_decision {
            parts.push(self.styler.green("[a]pprove"));
            parts.push(self.styler.dim("[d]eny"));
            parts.push(self.styler.dim("[q]uit"));
        } else {
            parts.push(self.styler.dim("[q]uit"));
        }
        parts.join("  ") + " >"
    }

    fn handle_navigation(&self, idx: usize, action: &Option<ViewerAction>) -> Option<usize> {
        if action.is_some() {
            return None;
        }
        if idx + 1 < self.files.len() {
            Some(idx + 1)
        } else if idx > 0 {
            Some(idx - 1)
        } else {
            Some(0)
        }
    }
}
