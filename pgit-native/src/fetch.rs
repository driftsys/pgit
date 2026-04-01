use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use pgit_core::purl::Purl;

/// Resolve the git clone URL from a PURL namespace.
///
/// Namespace format: `github.com/owner/repo` or `gitlab.com/owner/repo`
pub fn url_from_purl(purl: &Purl) -> Result<String> {
    let ns = purl
        .namespace
        .as_deref()
        .with_context(|| format!("PURL '{}' has no namespace (registry)", purl))?;

    // namespace = host/owner/repo — reconstruct HTTPS clone URL
    Ok(format!("https://{}.git", ns))
}

/// Shallow-clone a git repository into `dest/<dir_name>`.
///
/// Passes `GIT_LFS_SKIP_SMUDGE=0` so LFS pointers are smudged automatically
/// when git-lfs is available.
pub fn shallow_clone(
    url: &str,
    git_ref: Option<&str>,
    dir_name: &str,
    dest: &Path,
) -> Result<PathBuf> {
    let clone_dir = dest.join(dir_name);
    let clone_str = clone_dir
        .to_str()
        .with_context(|| "clone path is not valid UTF-8")?;

    let mut args: Vec<&str> = vec!["clone", "--depth", "1"];
    if let Some(r) = git_ref {
        args.extend_from_slice(&["--branch", r]);
    }
    args.push(url);
    args.push(clone_str);

    let output = Command::new("git")
        .args(&args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_LFS_SKIP_SMUDGE", "0")
        .output()
        .context("failed to run git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git clone failed: {}", stderr.trim());
    }

    Ok(clone_dir)
}

/// Resolve the full commit SHA of HEAD in a cloned repository.
pub fn resolve_head_sha(repo: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .env_remove("GIT_DIR")
        .output()
        .context("failed to run git rev-parse")?;

    if !output.status.success() {
        anyhow::bail!("git rev-parse failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Return the path to `subfolder` inside `clone_dir`, or an error if missing.
pub fn resolve_subfolder(clone_dir: &Path, subfolder: Option<&str>) -> Result<PathBuf> {
    match subfolder {
        None => Ok(clone_dir.to_path_buf()),
        Some(sub) => {
            let path = clone_dir.join(sub);
            anyhow::ensure!(
                path.is_dir(),
                "subfolder '{}' not found in cloned repo",
                sub
            );
            Ok(path)
        }
    }
}

/// Remove a temporary directory, ignoring errors.
pub fn cleanup(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}
