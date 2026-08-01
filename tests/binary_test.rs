use std::io::Write;
use std::process::{Command, Stdio};

fn run_statusline(session_json: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-statusline-rust"))
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
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = run_statusline(session_json);

    assert!(statusline.contains('⎇'));
}
