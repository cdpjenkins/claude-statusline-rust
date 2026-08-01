use claude_statusline_rust::{render, GitStatus};

#[test]
fn renders_the_statusline_for_a_session() {
    let session_json = include_str!("../initial-test-data/input.json");
    let expected = include_str!("../initial-test-data/expected.txt");

    let git_status = GitStatus {
        branch: "main".to_string(),
        dirty: true,
    };

    assert_eq!(render(session_json, &git_status), expected);
}
