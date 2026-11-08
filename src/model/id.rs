pub fn is_valid_package_id(id: &str) -> bool {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return false;
    }
    let segments: Vec<&str> = trimmed.split('.').collect();
    if segments.len() < 2 {
        return false;
    }
    for seg in segments {
        if seg.is_empty() {
            return false;
        }
        for c in seg.chars() {
            if !c.is_ascii_alphanumeric() && c != '-' {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ids() {
        assert!(is_valid_package_id("com.example.app"));
        assert!(is_valid_package_id("shareui.lacie"));
        assert!(is_valid_package_id("org.my-tool.v1"));
        assert!(!is_valid_package_id("single"));
        assert!(!is_valid_package_id("com..empty"));
        assert!(!is_valid_package_id("com.bad_char.app"));
    }
}
