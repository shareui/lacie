use crate::diag::DiagError;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub fn validate_safe_path(dest_dir: &Path, rel_path: &Path) -> Result<PathBuf, DiagError> {
    if rel_path.is_absolute() {
        let msg = rel_path.display().to_string();
        return Err(DiagError::from_code("E014", &[("ref", &msg)]));
    }
    for comp in rel_path.components() {
        if let Component::ParentDir = comp {
            let msg = rel_path.display().to_string();
            return Err(DiagError::from_code("E014", &[("ref", &msg)]));
        }
    }
    Ok(dest_dir.join(rel_path))
}

fn unpack_tar<R: Read>(mut archive: tar::Archive<R>, dest_dir: &Path) -> Result<(), DiagError> {
    let entries = archive.entries()
        .map_err(|e| DiagError::from_code("E014", &[("ref", &e.to_string())]))?;
    for entry_res in entries {
        let mut entry = entry_res
            .map_err(|e| DiagError::from_code("E014", &[("ref", &e.to_string())]))?;
        let entry_path = entry.path()
            .map_err(|e| DiagError::from_code("E014", &[("ref", &e.to_string())]))?
            .to_path_buf();
        let target = validate_safe_path(dest_dir, &entry_path)?;
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| DiagError::new("E015", e.to_string()))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DiagError::new("E015", e.to_string()))?;
            }
            entry.unpack(&target)
                .map_err(|e| DiagError::new("E015", e.to_string()))?;
        }
    }
    Ok(())
}

fn unpack_zip(path: &Path, dest_dir: &Path) -> Result<(), DiagError> {
    let file = File::open(path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| DiagError::from_code("E014", &[("ref", &e.to_string())]))?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)
            .map_err(|e| DiagError::from_code("E014", &[("ref", &e.to_string())]))?;
        let name = entry.name().to_string();
        let entry_path = PathBuf::from(&name);
        let target = validate_safe_path(dest_dir, &entry_path)?;
        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| DiagError::new("E015", e.to_string()))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DiagError::new("E015", e.to_string()))?;
            }
            let mut out = File::create(&target)
                .map_err(|e| DiagError::new("E015", e.to_string()))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| DiagError::new("E015", e.to_string()))?;
        }
    }
    Ok(())
}

fn unpack_tar_xz(path: &Path, dest_dir: &Path) -> Result<(), DiagError> {
    let file = File::open(path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let mut reader = std::io::BufReader::new(file);
    let mut decompressed = Vec::new();
    lzma_rs::xz_decompress(&mut reader, &mut decompressed)
        .map_err(|e| DiagError::from_code("E014", &[("ref", &e.to_string())]))?;
    let archive = tar::Archive::new(&decompressed[..]);
    unpack_tar(archive, dest_dir)
}

fn unpack_tar_zst(path: &Path, dest_dir: &Path) -> Result<(), DiagError> {
    let file = File::open(path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let decoder = zstd::stream::Decoder::new(file)
        .map_err(|e| DiagError::from_code("E014", &[("ref", &e.to_string())]))?;
    let archive = tar::Archive::new(decoder);
    unpack_tar(archive, dest_dir)
}

fn unpack_tar_gz(path: &Path, dest_dir: &Path) -> Result<(), DiagError> {
    let file = File::open(path)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let archive = tar::Archive::new(decoder);
    unpack_tar(archive, dest_dir)
}

pub fn unpack_archive(path: &Path, dest_dir: &Path) -> Result<(), DiagError> {
    std::fs::create_dir_all(dest_dir)
        .map_err(|e| DiagError::new("E015", e.to_string()))?;
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_lowercase();
    if filename.ends_with(".zip") {
        unpack_zip(path, dest_dir)
    } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        unpack_tar_gz(path, dest_dir)
    } else if filename.ends_with(".tar.xz") || filename.ends_with(".txz") {
        unpack_tar_xz(path, dest_dir)
    } else if filename.ends_with(".tar.zst") {
        unpack_tar_zst(path, dest_dir)
    } else {
        let msg = path.display().to_string();
        Err(DiagError::from_code("E014", &[("ref", &msg)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_path_traversal_rejection() {
        let base = Path::new("/var/lacie/unpack");
        assert!(validate_safe_path(base, Path::new("../evil.sh")).is_err());
        assert!(validate_safe_path(base, Path::new("/etc/passwd")).is_err());
        assert!(validate_safe_path(base, Path::new("sub/dir/app")).is_ok());
    }

    #[test]
    fn test_unpack_zip() {
        let tmp = std::env::temp_dir().join("lacie_test_zip");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let zip_path = tmp.join("test.zip");
        let dest = tmp.join("out");

        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("hello.txt", zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(b"hello world").unwrap();
        zip.finish().unwrap();

        unpack_archive(&zip_path, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("hello.txt")).unwrap(), "hello world");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_unpack_tar_gz() {
        let tmp = std::env::temp_dir().join("lacie_test_targz");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let tar_path = tmp.join("test.tar.gz");
        let dest = tmp.join("out");

        let file = File::create(&tar_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(gz);
        let mut header = tar::Header::new_gnu();
        header.set_size(8);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, "file.txt", b"hello gz" as &[u8]).unwrap();
        let gz = builder.into_inner().unwrap();
        gz.finish().unwrap();

        unpack_archive(&tar_path, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("file.txt")).unwrap(), "hello gz");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_unpack_tar_zst() {
        let tmp = std::env::temp_dir().join("lacie_test_tarzst");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let tar_path = tmp.join("test.tar.zst");
        let dest = tmp.join("out");

        let file = File::create(&tar_path).unwrap();
        let zst = zstd::stream::Encoder::new(file, 0).unwrap();
        let mut builder = tar::Builder::new(zst);
        let mut header = tar::Header::new_gnu();
        header.set_size(9);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, "file.txt", b"hello zst" as &[u8]).unwrap();
        let zst = builder.into_inner().unwrap();
        zst.finish().unwrap();

        unpack_archive(&tar_path, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("file.txt")).unwrap(), "hello zst");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_unpack_tar_xz() {
        let tmp = std::env::temp_dir().join("lacie_test_tarxz");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let tar_path = tmp.join("test.tar.xz");
        let dest = tmp.join("out");

        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(8);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, "file.txt", b"hello xz" as &[u8]).unwrap();
        let tar_buf = builder.into_inner().unwrap();

        let mut out_file = File::create(&tar_path).unwrap();
        lzma_rs::xz_compress(&mut &tar_buf[..], &mut out_file).unwrap();

        unpack_archive(&tar_path, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("file.txt")).unwrap(), "hello xz");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
