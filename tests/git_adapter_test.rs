use claude_statusline_rust::git;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryRepo {
    path: PathBuf,
}

impl TemporaryRepo {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("statusline-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let repo = TemporaryRepo { path };
        repo.run(&["init", "--initial-branch=main"]);
        repo.run(&["config", "user.email", "test@example.com"]);
        repo.run(&["config", "user.name", "Test"]);
        repo
    }

    fn run(&self, arguments: &[&str]) {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(&self.path)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {:?} failed", arguments);
    }

    fn write_file(&self, name: &str, contents: &str) {
        fs::write(self.path.join(name), contents).unwrap();
    }

    fn commit_everything(&self, message: &str) {
        self.run(&["add", "-A"]);
        self.run(&["commit", "-m", message]);
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn reads_the_branch_of_a_clean_repository() {
    let repo = TemporaryRepo::new("clean");
    repo.write_file("file.txt", "committed");
    repo.commit_everything("initial");

    let status = git::read(repo.path()).unwrap();

    assert_eq!(status.branch, "main");
    assert!(!status.dirty);
}

#[test]
fn reports_a_repository_with_uncommitted_changes_as_dirty() {
    let repo = TemporaryRepo::new("dirty");
    repo.write_file("file.txt", "committed");
    repo.commit_everything("initial");
    repo.write_file("file.txt", "modified");

    let status = git::read(repo.path()).unwrap();

    assert_eq!(status.branch, "main");
    assert!(status.dirty);
}

#[test]
fn reports_an_untracked_file_as_dirty() {
    let repo = TemporaryRepo::new("untracked");
    repo.write_file("file.txt", "committed");
    repo.commit_everything("initial");
    repo.write_file("extra.txt", "untracked");

    let status = git::read(repo.path()).unwrap();

    assert!(status.dirty);
}

#[test]
fn reads_the_current_branch_after_switching() {
    let repo = TemporaryRepo::new("branch");
    repo.write_file("file.txt", "committed");
    repo.commit_everything("initial");
    repo.run(&["checkout", "-b", "feature/thing"]);

    let status = git::read(repo.path()).unwrap();

    assert_eq!(status.branch, "feature/thing");
}

#[test]
fn shows_the_short_commit_when_head_is_detached() {
    let repo = TemporaryRepo::new("detached");
    repo.write_file("file.txt", "committed");
    repo.commit_everything("initial");
    repo.run(&["checkout", "--detach"]);

    let status = git::read(repo.path()).unwrap();

    assert_ne!(status.branch, "HEAD");
    assert_eq!(status.branch.len(), 7);
}

#[test]
fn has_no_status_outside_a_repository() {
    let outside = std::env::temp_dir().join(format!("statusline-bare-{}", std::process::id()));
    let _ = fs::remove_dir_all(&outside);
    fs::create_dir_all(&outside).unwrap();

    assert!(git::read(&outside).is_none());

    let _ = fs::remove_dir_all(&outside);
}
