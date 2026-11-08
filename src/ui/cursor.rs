use std::io::{stdout, Write};

pub fn hide_cursor() {
    print!("\x1b[?25l");
    let _ = stdout().flush();
}

pub fn show_cursor() {
    print!("\x1b[?25h");
    let _ = stdout().flush();
}

pub struct CursorGuard;

impl Default for CursorGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorGuard {
    pub fn new() -> Self {
        hide_cursor();
        Self
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        show_cursor();
    }
}
