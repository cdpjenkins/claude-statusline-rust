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
fn omits_the_model_segment_when_the_model_is_missing() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("project"));
    assert!(!statusline.contains('✱'));
}

#[test]
fn omits_the_token_segments_when_the_context_window_is_missing() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"},
        "model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains("tokens"));
    assert!(!statusline.contains('◔'));
}

#[test]
fn omits_the_directory_segment_when_the_workspace_is_missing() {
    let session_json = r#"{"model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("Opus 5"));
}

#[test]
fn warns_when_the_session_cannot_be_parsed() {
    let statusline = render("not json", None);

    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn warns_when_the_session_has_nothing_to_show() {
    let statusline = render("{}", None);

    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn omits_the_git_segment_outside_a_repository() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = render(session_json, None);

    assert!(statusline.contains("initial-test-data"));
    assert!(!statusline.contains('⎇'));
}
