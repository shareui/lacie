pub mod color;
pub mod cursor;
pub mod diff;
pub mod progress;
pub mod prompt;
pub mod syntax;
pub mod viewer;

use std::path::Path;

pub fn shorten_bin_path_display(path: &Path) -> String {
    let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
        return path.display().to_string();
    };
    let Some(parent) = path.parent() else {
        return path.display().to_string();
    };

    if let Some((prefix, suffix)) = file_name.rsplit_once('-') {
        let (hash, ext) = match suffix.split_once('.') {
            Some((h, ext)) => (h, std::format!(".{}", ext)),
            None => (suffix, String::new()),
        };
        if hash.len() > 6 {
            let short_hash = std::format!("{}...{}{}", &hash[..3], &hash[hash.len() - 3..], ext);
            let short_file = std::format!("{}-{}", prefix, short_hash);
            return parent.join(short_file).display().to_string();
        }
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_bin_path_display() {
        let p = Path::new("/home/shareui/lacie/bin/de.shareui.yukinolsp/0.1.0/yukino-lsp-08cd27755a43329f5756f86f20b9ef9944d877102efc37d1df201511c90ccc5c");
        let shortened = shorten_bin_path_display(p);
        assert_eq!(
            shortened,
            "/home/shareui/lacie/bin/de.shareui.yukinolsp/0.1.0/yukino-lsp-08c...c5c"
        );

        let p_exe = Path::new("/bin/tool-08cd27755a43329f5756f86f20b9ef9944d877102efc37d1df201511c90ccc5c.exe");
        assert_eq!(shorten_bin_path_display(p_exe), "/bin/tool-08c...c5c.exe");

        let p_short = Path::new("/bin/tool-12345");
        assert_eq!(shorten_bin_path_display(p_short), "/bin/tool-12345");
    }
}

