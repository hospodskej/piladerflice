//! Finding the files and naming them by their content.

use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

/// Every file under `dir`, as (name, file), sorted by name. Hidden files
/// (`.gitkeep`, `.DS_Store`) are left out.
pub(crate) fn walk(root: &Path, dir: &Path, prefix: &str, found: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
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

/// The name with a fingerprint of the content: `"css/site.css"` becomes
/// `"css/site-1a2b3c4d.css"` (the first 8 hex digits of its SHA-1).
///
/// ```
/// let name = sapwood_assets::fingerprinted_path("css/site.css", b"body {}");
/// assert!(name.starts_with("css/site-") && name.ends_with(".css"));
/// ```
pub fn fingerprinted_path(name: &str, content: &[u8]) -> String {
    let digest = hex::encode(Sha1::digest(content));
    let fingerprint = &digest[..8];
    let (dir, file) = name.rsplit_once('/').map_or(("", name), |(dir, file)| (dir, file));
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
        let name = fingerprinted_path("base/global.css", b"body{}");
        assert!(name.starts_with("base/global-"), "{name}");
        assert!(name.ends_with(".css"));
        assert_eq!(name.len(), "base/global-12345678.css".len());
        assert!(fingerprinted_path("turbo.min.js", b"x").starts_with("turbo.min-"));
        assert!(fingerprinted_path("LICENSE", b"x").starts_with("LICENSE-"));
        assert_ne!(fingerprinted_path("a.css", b"1"), fingerprinted_path("a.css", b"2"));
    }
}
