use crate::diag::DiagError;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn compute_sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    std::format!("{:x}", hasher.finalize())
}

pub fn compute_sha256_file(path: &Path) -> Result<String, DiagError> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer)
            .map_err(|e| DiagError::new("E015", e.to_string()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(std::format!("{:x}", hasher.finalize()))
}

pub fn normalize_lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                i += 1;
            }
            out.push(b'\n');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    out
}

pub fn compute_sha256_lf(bytes: &[u8]) -> String {
    let normalized = normalize_lf(bytes);
    compute_sha256_bytes(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_lf_normalization() {
        let crlf = b"name = \"lacie\"\r\nversion = \"0.1.0\"\r\n";
        let lf = b"name = \"lacie\"\nversion = \"0.1.0\"\n";
        assert_eq!(compute_sha256_lf(crlf), compute_sha256_lf(lf));
    }
}
