//! Finding the asset files and naming them by their content.

use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

/// Every file under `dir`, as (logical path, file), sorted by name.
pub fn walk(root: &Path, dir: &Path, prefix: &str, found: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, prefix, found)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            found.push((format!("{prefix}{relative}"), path));
        }
    }
    Ok(())
}

/// "base/global.css" + content -> "base/global-1a2b3c4d.css"
pub fn digested_path(logical: &str, content: &[u8]) -> String {
    let digest = hex::encode(Sha1::digest(content));
    let fingerprint = &digest[..8];
    let (dir, file) = logical
        .rsplit_once('/')
        .map_or(("", logical), |(dir, file)| (dir, file));
    let renamed = match file.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}-{fingerprint}.{ext}"),
        None => format!("{file}-{fingerprint}"),
    };
    if dir.is_empty() {
        renamed
    } else {
        format!("{dir}/{renamed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_keep_the_folder_and_extension() {
        let digested = digested_path("base/global.css", b"body{}");
        assert!(digested.starts_with("base/global-"), "{digested}");
        assert!(digested.ends_with(".css"));
        assert_eq!(digested.len(), "base/global-12345678.css".len());
        assert!(digested_path("turbo.min.js", b"x").starts_with("turbo.min-"));
    }
}
