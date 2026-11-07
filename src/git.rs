use crate::GitStatus;
use std::path::Path;
use std::process::Command;

pub fn read(directory: &Path) -> Option<GitStatus> {
    let branch = current_branch(directory)?;
    let dirty = has_uncommitted_changes(directory)?;

    Some(GitStatus { branch, dirty })
}

fn current_branch(directory: &Path) -> Option<String> {
    let branch = run_git(directory, &["rev-parse", "--abbrev-ref", "HEAD"])?;

    if branch == "HEAD" {
        run_git(directory, &["rev-parse", "--short=7", "HEAD"])
    } else {
        Some(branch)
    }
}

fn has_uncommitted_changes(directory: &Path) -> Option<bool> {
    // Untracked files count whatever status.showUntrackedFiles says.
    let changes = run_git(
        directory,
        &["status", "--porcelain", "--untracked-files=normal"],
    )?;

    Some(!changes.is_empty())
}

fn run_git(directory: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;

    Some(text.trim().to_string())
}
