use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Stage all changes, commit with `message`, and push to `remote`.
pub fn commit_and_push(repo: &Path, message: &str, remote: &str) -> Result<()> {
    git(repo, &["add", "."])?;
    git(repo, &[
        "-c", "user.name=pgit",
        "-c", "user.email=pgit@users.noreply.github.com",
        "commit", "--allow-empty", "-m", message,
    ])?;
    git(repo, &["push", remote, "HEAD"])?;
    Ok(())
}

/// Create an annotated tag and push it.
pub fn tag_and_push(repo: &Path, tag: &str, message: &str, remote: &str) -> Result<()> {
    git(repo, &["tag", "-a", tag, "-m", message])?;
    git(repo, &["push", remote, tag])?;
    Ok(())
}

fn git(repo: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .context("failed to run git")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {:?} failed: {}", args, stderr.trim());
    }
    Ok(())
}
