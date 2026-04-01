use std::path::Path;

use anyhow::{Context, Result};
use pgit_core::hash;

/// Recursively copy `src` into `dest`, skipping `.git`.
pub fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)
        .with_context(|| format!("create {}", dest.display()))?;

    for entry in std::fs::read_dir(src)
        .with_context(|| format!("read {}", src.display()))?
    {
        let entry = entry.with_context(|| format!("read entry in {}", src.display()))?;
        if entry.file_name() == ".git" {
            continue;
        }
        let src_path  = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path).with_context(|| {
                format!("copy {} -> {}", src_path.display(), dest_path.display())
            })?;
        }
    }
    Ok(())
}

/// Collect all files under `root` as `(relative_path, bytes)` pairs,
/// skipping `.git`.  Used for tree hashing.
pub fn collect_tree(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut entries = Vec::new();
    collect_recursive(root, root, &mut entries)?;
    Ok(entries)
}

fn collect_recursive(
    root:    &Path,
    current: &Path,
    out:     &mut Vec<(String, Vec<u8>)>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)
        .with_context(|| format!("read {}", current.display()))?
    {
        let entry = entry?;
        if entry.file_name() == ".git" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&path)
                .with_context(|| format!("read {}", path.display()))?;
            out.push((rel, bytes));
        }
    }
    Ok(())
}

/// Compute the SHA-256 tree hash of a directory.
pub fn hash_dir(root: &Path) -> Result<String> {
    let mut entries = collect_tree(root)?;
    Ok(hash::sha256_tree(&mut entries))
}
