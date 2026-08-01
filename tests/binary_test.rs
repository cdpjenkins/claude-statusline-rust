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
fn writes_the_statusline_for_a_session_on_stdin() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = run_statusline(session_json);
    let lines: Vec<&str> = statusline.lines().collect();

    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("initial-test-data"));
    assert!(lines[1].contains("Opus 5"));
    assert!(lines[1].contains("20.8K tokens"));
    assert!(lines[2].contains("20,812 (98%)"));
}

#[test]
fn reads_git_state_from_the_session_directory() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = run_statusline(session_json);
    let directory_line = statusline.lines().next().unwrap();

    assert!(directory_line.contains('⎇'));
}
