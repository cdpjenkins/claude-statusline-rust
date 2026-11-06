use claude_statusline_rust::{render, GitStatus};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 2h14m before the five hour reset in `initial-test-data/input.json`.
const NOW_SECONDS: u64 = 1_785_595_200 - (2 * 3600 + 14 * 60);
fn now() -> SystemTime {
    at(NOW_SECONDS)
}

fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[test]
fn renders_the_statusline_for_a_session() {
    let session_json = include_str!("../initial-test-data/input.json");
    let expected = include_str!("../initial-test-data/expected.txt");

    let git_status = GitStatus {
        branch: "main".to_string(),
        dirty: true,
    };

    assert_eq!(render(session_json, Some(&git_status), now()), expected);
}

#[test]
fn marks_the_line_as_rendered_by_the_rust_statusline() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains('🦀'));
}

#[test]
fn puts_the_rust_marker_before_every_other_segment() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"},
        "model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None, now());

    let marker = statusline.find('🦀').unwrap();
    let directory = statusline.find("project").unwrap();

    assert!(marker < directory);
}

#[test]
fn colours_the_rust_marker_orange() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("\x1b[48;5;202m\x1b[38;5;231m 🦀 "));
}

#[test]
fn omits_the_model_segment_when_the_model_is_missing() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("project"));
    assert!(!statusline.contains('✱'));
}

#[test]
fn omits_the_context_segment_when_the_context_window_is_missing() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"},
        "model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains('◔'));
}

#[test]
fn omits_the_directory_segment_when_the_workspace_is_missing() {
    let session_json = r#"{"model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("Opus 5"));
}

#[test]
fn warns_when_the_session_cannot_be_parsed() {
    let statusline = render("not json", None, now());

    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn warns_when_the_session_has_nothing_to_show() {
    let statusline = render("{}", None, now());

    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn marks_the_warning_line_as_rendered_by_the_rust_statusline() {
    let statusline = render("not json", None, now());

    assert!(statusline.contains('🦀'));
    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn puts_the_rust_marker_before_the_warning() {
    let statusline = render("not json", None, now());

    let marker = statusline.find('🦀').unwrap();
    let warning = statusline.find('⚠').unwrap();

    assert!(marker < warning);
}

#[test]
fn marks_an_empty_session_as_rendered_by_the_rust_statusline() {
    let statusline = render("{}", None, now());

    assert!(statusline.contains('🦀'));
    assert!(statusline.contains("⚠ statusline"));
}

#[test]
fn renders_both_rate_limit_windows() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33},
        "seven_day":{"used_percentage":4}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("◷ 33%"));
    assert!(statusline.contains("▦ 4%"));
}

#[test]
fn omits_the_rate_limits_when_the_session_does_not_report_them() {
    let session_json = r#"{"model":{"display_name":"Opus 5"}}"#;

    let statusline = render(session_json, None, now());

    assert!(!statusline.contains('◷'));
    assert!(!statusline.contains('▦'));
}

#[test]
fn omits_the_rate_limits_when_the_windows_are_missing() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},"rate_limits":{}}"#;

    let statusline = render(session_json, None, now());

    assert!(!statusline.contains('◷'));
    assert!(!statusline.contains('▦'));
}

#[test]
fn renders_only_the_window_the_session_reports() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("◷ 33%"));
    assert!(!statusline.contains('▦'));
}

#[test]
fn renders_an_unused_window_as_zero_rather_than_omitting_it() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":0},
        "seven_day":{"used_percentage":0}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("◷ 0%"));
    assert!(statusline.contains("▦ 0%"));
}

#[test]
fn orders_the_rate_limits_after_the_context_and_shortest_window_first() {
    let session_json = r#"{"context_window":{"total_input_tokens":20812,"remaining_percentage":98},
        "rate_limits":{"five_hour":{"used_percentage":33},"seven_day":{"used_percentage":4}}}"#;

    let statusline = render(session_json, None, now());

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

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("\x1b[48;5;60m\x1b[38;5;231m ◷ 33% "));
    assert!(statusline.contains("\x1b[48;5;66m\x1b[38;5;231m ▦ 4% "));
}

#[test]
fn keeps_every_other_segment_when_a_percentage_is_not_a_number() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project"},
        "model":{"display_name":"Opus 5"},
        "rate_limits":{"five_hour":{"used_percentage":"lots"}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("project"));
    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains("⚠ statusline"));
}

#[test]
fn keeps_every_other_segment_when_a_window_reports_no_percentage() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},
        "rate_limits":{"five_hour":{}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains('◷'));
    assert!(!statusline.contains("⚠ statusline"));
}

#[test]
fn renders_the_context_percentage_when_it_arrives_as_a_decimal() {
    let session_json = r#"{"context_window":{"total_input_tokens":20812,
        "remaining_percentage":98.4}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("◔ 20,812 (98%)"));
}

#[test]
fn keeps_every_other_segment_when_a_token_count_is_not_a_number() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},
        "context_window":{"total_input_tokens":"lots","remaining_percentage":98}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains('◔'));
    assert!(!statusline.contains("⚠ statusline"));
}

#[test]
fn renders_a_rate_limit_percentage_when_it_arrives_as_a_decimal() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33.4}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("◷ 33%"));
}

const SECTIONS: [(&str, &str, &str); 4] = [
    ("model", r#"{"display_name":"Opus 5"}"#, "Opus 5"),
    ("workspace", r#"{"current_dir":"/x/project"}"#, "project"),
    (
        "context_window",
        r#"{"total_input_tokens":20812,"remaining_percentage":98}"#,
        "20,812",
    ),
    (
        "rate_limits",
        r#"{"five_hour":{"used_percentage":33}}"#,
        "33%",
    ),
];

fn session_with_broken(broken: &str) -> String {
    let sections: Vec<String> = SECTIONS
        .iter()
        .map(|(name, value, _)| {
            let value = if *name == broken {
                r#""unexpected""#
            } else {
                value
            };

            format!(r#""{}":{}"#, name, value)
        })
        .collect();

    format!("{{{}}}", sections.join(","))
}

#[test]
fn keeps_the_other_sections_when_one_is_not_an_object() {
    for (broken, _, _) in SECTIONS {
        let statusline = render(&session_with_broken(broken), None, now());

        assert!(!statusline.contains("⚠ statusline"), "broken: {}", broken);
        for (name, _, marker) in SECTIONS {
            if name != broken {
                assert!(statusline.contains(marker), "{} lost {}", broken, marker);
            }
        }
    }
}

#[test]
fn costs_only_its_own_segment_when_a_name_is_not_a_string() {
    let session_json = r#"{"model":{"display_name":42},
        "workspace":{"current_dir":"/x/project"}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("project"));
    assert!(!statusline.contains('✱'));
    assert!(!statusline.contains("⚠ statusline"));
}

#[test]
fn drops_a_negative_percentage_rather_than_showing_it_as_zero() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},
        "rate_limits":{"five_hour":{"used_percentage":-5}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains('◷'));
}

#[test]
fn drops_a_token_count_too_large_to_represent() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},
        "context_window":{"total_input_tokens":1e300,"remaining_percentage":98}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains('◔'));
}

#[test]
fn keeps_every_other_segment_when_a_percentage_exceeds_a_byte() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},
        "rate_limits":{"five_hour":{"used_percentage":300}}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("Opus 5"));
    assert!(!statusline.contains('◷'));
    assert!(!statusline.contains("⚠ statusline"));
}

#[test]
fn omits_the_git_segment_outside_a_repository() {
    let session_json = include_str!("../initial-test-data/input.json");

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("initial-test-data"));
    assert!(!statusline.contains('⎇'));
}

#[test]
fn shows_how_long_until_each_rate_limit_resets() {
    let session_json = r#"{"rate_limits":{
        "five_hour":{"used_percentage":33,"resets_at":10800},
        "seven_day":{"used_percentage":4,"resets_at":277200}}}"#;

    let statusline = render(session_json, None, at(0));

    assert!(statusline.contains(" ◷ 33% 3h "));
    assert!(statusline.contains(" ▦ 4% 3d5h "));
}

#[test]
fn shows_only_the_percentage_when_the_reset_time_is_missing() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33}}}"#;

    let statusline = render(session_json, None, at(0));

    assert!(statusline.contains(" ◷ 33% "));
}

#[test]
fn shows_only_the_percentage_when_the_reset_time_is_not_a_number() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33,"resets_at":"soon"}}}"#;

    let statusline = render(session_json, None, at(0));

    assert!(statusline.contains(" ◷ 33% "));
}

#[test]
fn omits_the_countdown_once_the_reset_has_passed() {
    let session_json = r#"{"rate_limits":{"five_hour":{"used_percentage":33,"resets_at":1000}}}"#;

    assert!(render(session_json, None, at(1000)).contains(" ◷ 33% "));
    assert!(render(session_json, None, at(5000)).contains(" ◷ 33% "));
}

#[test]
fn drops_the_rate_limit_segment_when_only_the_reset_time_is_present() {
    let session_json = r#"{"model":{"display_name":"Opus 5"},
        "rate_limits":{"five_hour":{"resets_at":10800}}}"#;

    let statusline = render(session_json, None, at(0));

    assert!(!statusline.contains('◷'));
}

#[test]
fn names_the_directory_when_the_path_ends_in_a_slash() {
    let session_json = r#"{"workspace":{"current_dir":"/x/project/"}}"#;

    let statusline = render(session_json, None, now());

    assert!(statusline.contains("\x1b[48;5;136m\x1b[38;5;231m project "));
}

#[test]
fn omits_the_directory_segment_when_the_path_has_no_name() {
    for current_dir in ["/", "", "//"] {
        let session_json = format!(
            r#"{{"workspace":{{"current_dir":"{}"}},"model":{{"display_name":"Opus 5"}}}}"#,
            current_dir
        );

        let statusline = render(&session_json, None, now());

        assert!(
            statusline.contains("Opus 5"),
            "current_dir: {:?}",
            current_dir
        );
        assert!(
            !statusline.contains("\x1b[48;5;136m\x1b[38;5;231m"),
            "current_dir: {:?}",
            current_dir
        );
    }
}
