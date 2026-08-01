use claude_statusline_rust::{render, GitStatus};

#[test]
fn renders_the_statusline_for_a_session() {
    let session_json = include_str!("../initial-test-data/input.json");
    let expected = include_str!("../initial-test-data/expected.txt");

    let git_status = GitStatus {
        branch: "main".to_string(),
        dirty: true,
    };

    assert_eq!(render(session_json, Some(&git_status)), expected);
}

#[test]
fn omits_the_git_segment_outside_a_repository() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = render(session_json, None);
    let directory_line = statusline.lines().next().unwrap();

    assert!(directory_line.contains("initial-test-data"));
    assert!(!directory_line.contains('⎇'));
}
