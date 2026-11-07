mod support;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use support::TemporaryRepo;

fn run_statusline(session_json: &str) -> String {
    run_statusline_with(session_json, &[], std::env::temp_dir().as_path())
}

fn run_statusline_with(session_json: &str, arguments: &[&str], home: &Path) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-statusline-rust"))
        .args(arguments)
        .env("HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(session_json.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "statusline exited with failure");

    String::from_utf8(output.stdout).unwrap()
}

struct TemporaryHome {
    path: PathBuf,
}

impl TemporaryHome {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("statusline-home-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        TemporaryHome { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn saved_input(&self) -> PathBuf {
        self.path.join("claude-statusline-input.jsonl")
    }

    fn saved_lines(&self) -> Vec<String> {
        fs::read_to_string(self.saved_input())
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

impl Drop for TemporaryHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn warns_instead_of_failing_when_the_session_is_not_json() {
    let statusline = run_statusline("this is not json");

    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn warns_instead_of_failing_when_stdin_is_empty() {
    let statusline = run_statusline("");

    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn warns_instead_of_failing_when_the_session_is_truncated() {
    let session_json = include_str!("../initial-test-data/input.json");
    let truncated = &session_json[..session_json.len() / 2];

    let statusline = run_statusline(truncated);

    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn writes_the_statusline_for_a_session_on_stdin() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = run_statusline(session_json);
    let lines: Vec<&str> = statusline.lines().collect();

    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("initial-test-data"));
    assert!(lines[0].contains("Opus 5"));
    assert!(lines[0].contains("20,812 (98%)"));
}

#[test]
fn ends_the_statusline_with_a_single_newline() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = run_statusline(session_json);

    assert_eq!(statusline.matches('\n').count(), 1);
    assert!(statusline.ends_with('\n'));
}

#[test]
fn reads_git_state_from_the_session_directory() {
    let repo = TemporaryRepo::new("binary");
    repo.write_file("file.txt", "committed");
    repo.commit_everything("initial");
    let session_json = serde_json::json!({
        "workspace": {"current_dir": repo.path()},
    })
    .to_string();

    let statusline = run_statusline(&session_json);

    assert!(statusline.contains("⎇ main "));
}

#[test]
fn saves_the_session_when_asked_to() {
    let session_json = include_str!("../initial-test-data/input.json");
    let home = TemporaryHome::new("saves");

    run_statusline_with(session_json, &["--save-input"], home.path());

    let lines = home.saved_lines();
    assert_eq!(lines.len(), 1);
    let saved: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(
        saved,
        serde_json::from_str::<serde_json::Value>(session_json).unwrap()
    );
}

#[test]
fn saves_nothing_without_the_flag() {
    let session_json = include_str!("../initial-test-data/input.json");
    let home = TemporaryHome::new("unsaved");

    run_statusline_with(session_json, &[], home.path());

    assert!(!home.saved_input().exists());
}

#[test]
fn appends_a_line_for_every_run() {
    let session_json = include_str!("../initial-test-data/input.json");
    let home = TemporaryHome::new("repeated");

    run_statusline_with(session_json, &["--save-input"], home.path());
    run_statusline_with(session_json, &["--save-input"], home.path());

    assert_eq!(home.saved_lines().len(), 2);
}

#[test]
fn saves_a_session_it_cannot_parse() {
    let home = TemporaryHome::new("unparseable");

    let statusline = run_statusline_with("this is not json", &["--save-input"], home.path());

    assert_eq!(home.saved_lines(), vec!["this is not json"]);
    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn renders_the_same_line_whether_or_not_the_session_is_saved() {
    let session_json = include_str!("../initial-test-data/input.json");
    let home = TemporaryHome::new("unchanged");

    let saved = run_statusline_with(session_json, &["--save-input"], home.path());
    let unsaved = run_statusline_with(session_json, &[], home.path());

    assert_eq!(saved, unsaved);
}
