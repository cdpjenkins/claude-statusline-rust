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
fn marks_the_line_as_rendered_by_the_rust_statusline() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains('🦀'));
}

#[test]
fn puts_the_rust_marker_before_every_other_segment() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"},
        "model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None);

    let marker = statusline.find('🦀').unwrap();
    let directory = statusline.find("project").unwrap();

    assert!(marker < directory);
}

#[test]
fn colours_the_rust_marker_orange() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("\x1b[48;5;202m\x1b[38;5;231m 🦀 "));
}

#[test]
fn omits_the_model_segment_when_the_model_is_missing() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("project"));
    assert!(!statusline.contains('✱'));
}

#[test]
fn omits_the_context_segment_when_the_context_window_is_missing() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"},
        "model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("Opus 5"));
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
fn marks_the_warning_line_as_rendered_by_the_rust_statusline() {
    let statusline = render("not json", None);

    assert!(statusline.contains('🦀'));
    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn puts_the_rust_marker_before_the_warning() {
    let statusline = render("not json", None);

    let marker = statusline.find('🦀').unwrap();
    let warning = statusline.find('⚠').unwrap();

    assert!(marker < warning);
}

#[test]
fn marks_an_empty_session_as_rendered_by_the_rust_statusline() {
    let statusline = render("{}", None);

    assert!(statusline.contains('🦀'));
    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn renders_both_rate_limit_windows() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33},
        "seven_day":{"used_percentage":4}}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("◷ 33%"));
    assert!(statusline.contains("▦ 4%"));
}

#[test]
fn omits_the_rate_limits_when_the_session_does_not_report_them() {
    let session_json = r#"{"model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None);

    assert!(!statusline.contains('◷'));
    assert!(!statusline.contains('▦'));
}

#[test]
fn omits_the_rate_limits_when_the_windows_are_missing() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},"rate_limits":{}}"#;

    let statusline = render(session_json, None);

    assert!(!statusline.contains('◷'));
    assert!(!statusline.contains('▦'));
}

#[test]
fn renders_only_the_window_the_session_reports() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33}}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("◷ 33%"));
    assert!(!statusline.contains('▦'));
}

#[test]
fn renders_an_unused_window_as_zero_rather_than_omitting_it() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":0},
        "seven_day":{"used_percentage":0}}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("◷ 0%"));
    assert!(statusline.contains("▦ 0%"));
}

#[test]
fn orders_the_rate_limits_after_the_context_and_shortest_window_first() {
    let session_json = r#"{"context_window":{"total_input_tokens":20812,"remaining_percentage":98},
        "rate_limits":{"five_hour":{"used_percentage":33},"seven_day":{"used_percentage":4}}}"#;

    let statusline = render(session_json, None);

    let context = statusline.find("20,812").unwrap();
    let five_hour = statusline.find("◷ 33%").unwrap();
    let seven_day = statusline.find("▦ 4%").unwrap();

    assert!(context < five_hour);
    assert!(five_hour < seven_day);
}

#[test]
fn colours_each_rate_limit_window_distinctly() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33},
        "seven_day":{"used_percentage":4}}}"#;

    let statusline = render(session_json, None);

    assert!(statusline.contains("\x1b[48;5;60m\x1b[38;5;231m ◷ 33% "));
    assert!(statusline.contains("\x1b[48;5;66m\x1b[38;5;231m ▦ 4% "));
}

#[test]
fn omits_the_git_segment_outside_a_repository() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = render(session_json, None);

    assert!(statusline.contains("initial-test-data"));
    assert!(!statusline.contains('⎇'));
}
