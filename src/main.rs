use claude_statusline_rust::args::Arguments;
use claude_statusline_rust::{git, input_log, render, Session};
use std::io::Read;
use std::path::{Path, PathBuf};

const SAVED_INPUT_FILE: &str = "claude-statusline-input.jsonl";

fn main() {
    let arguments = Arguments::parse(std::env::args());

    let mut session_json = String::new();
    let _ = std::io::stdin().read_to_string(&mut session_json);

    if arguments.save_input {
        if let Some(path) = saved_input_path() {
            input_log::append(&path, &session_json);
        }
    }

    let current_dir = Session::parse(&session_json)
        .and_then(|session| session.current_directory().map(str::to_string));

    let git_status = current_dir.and_then(|dir| git::read(Path::new(&dir)));

    print!("{}", render(&session_json, git_status.as_ref()));
}

fn saved_input_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;

    Some(Path::new(&home).join(SAVED_INPUT_FILE))
}
