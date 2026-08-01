use claude_statusline_rust::git::GitInfo;
use claude_statusline_rust::status_line::render;

struct FakeGit {
    description: String,
}

impl GitInfo for FakeGit {
    fn describe(&self) -> Option<String> {
        Some(self.description.clone())
    }
}

#[test]
fn renders_a_powerline_status_line_from_claude_code_session_json() {
    let input = include_str!("../initial-test-data/input.json");
    let git = FakeGit {
        description: "No commits yet on main ●".to_string(),
    };

    let line = render(input, &git);

    let expected = concat!(
        "\x1b[0m\x1b[48;5;136m\x1b[38;5;231m claude-statusline-rust ",
        "\x1b[0m\x1b[48;5;238m\x1b[38;5;136m\u{e0b0}\x1b[48;5;238m\x1b[38;5;231m ⎇ No commits yet on main ● ",
        "\x1b[0m\x1b[48;5;237m\x1b[38;5;238m\u{e0b0}\x1b[48;5;237m\x1b[38;5;231m ✱ Sonnet 5 ",
        "\x1b[0m\x1b[48;5;235m\x1b[38;5;237m\u{e0b0}\x1b[48;5;235m\x1b[38;5;51m § 0 tokens ",
        "\x1b[0m\x1b[48;5;234m\x1b[38;5;235m\u{e0b0}\x1b[48;5;234m\x1b[38;5;157m ☉ $0.00 5% ",
        "\x1b[0m\x1b[48;5;240m\x1b[38;5;234m\u{e0b0}\x1b[48;5;240m\x1b[38;5;251m ◔ 0 (100%) ",
        "\x1b[0m\x1b[38;5;240m\u{e0b0}\x1b[0m",
    );

    assert_eq!(line, expected);
}
