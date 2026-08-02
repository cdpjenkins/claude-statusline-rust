use claude_statusline_rust::args::Arguments;

fn parse(arguments: &[&str]) -> Arguments {
    Arguments::parse(arguments.iter().map(|argument| argument.to_string()))
}

#[test]
fn does_not_save_the_input_without_the_flag() {
    let arguments = parse(&["claude-statusline-rust"]);

    assert!(!arguments.save_input);
}

#[test]
fn saves_the_input_when_the_flag_is_given() {
    let arguments = parse(&["claude-statusline-rust", "--save-input"]);

    assert!(arguments.save_input);
}

#[test]
fn ignores_an_unrecognised_argument() {
    let arguments = parse(&["claude-statusline-rust", "--wat"]);

    assert!(!arguments.save_input);
}

#[test]
fn recognises_the_flag_beside_an_unrecognised_argument() {
    let arguments = parse(&["claude-statusline-rust", "--wat", "--save-input"]);

    assert!(arguments.save_input);
}

#[test]
fn ignores_the_program_name() {
    let arguments = parse(&["--save-input"]);

    assert!(!arguments.save_input);
}
