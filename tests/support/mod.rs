//! Helpers shared by the integration tests. Each test file that uses them
//! declares `mod support;`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A git repository in the temporary directory, removed when dropped. It sets
/// its own identity and ignores the machine's global and system git config,
/// so it works the same on a machine with no config and on one whose config
/// signs commits, runs hooks or supplies a commit template.
pub struct TemporaryRepo {
    path: PathBuf,
}

impl TemporaryRepo {
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("statusline-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let repo = TemporaryRepo { path };
        repo.run(&["init", "--initial-branch=main"]);
        repo.run(&["config", "user.email", "test@example.com"]);
        repo.run(&["config", "user.name", "Test"]);
        repo
    }

    pub fn run(&self, arguments: &[&str]) {
        let output = Command::new("git")
            .args(arguments)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .current_dir(&self.path)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {:?} failed", arguments);
    }

    pub fn write_file(&self, name: &str, contents: &str) {
        fs::write(self.path.join(name), contents).unwrap();
    }

    pub fn commit_everything(&self, message: &str) {
        self.run(&["add", "-A"]);
        self.run(&["commit", "-m", message]);
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
