use claude_statusline_rust::args::Arguments;
use claude_statusline_rust::{git, input_log, render, Session};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const SAVED_INPUT_FILE: &str = "claude-statusline-input.jsonl";

fn main() {
    let arguments = Arguments::parse(std::env::args());

    let session_json = read_session_from_stdin();

    if arguments.save_input {
        if let Some(path) = saved_input_path() {
            input_log::append(&path, &session_json);
        }
    }

    let session = Session::parse(&session_json);

    let git_status = session
        .as_ref()
        .and_then(Session::current_directory)
        .and_then(|dir| git::read(Path::new(dir)));

    print!(
        "{}",
        render(session.as_ref(), git_status.as_ref(), SystemTime::now())
    );
}

fn read_session_from_stdin() -> String {
    let mut session_json = String::new();
    let _ = std::io::stdin().read_to_string(&mut session_json);
    session_json
}

fn saved_input_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;

    Some(Path::new(&home).join(SAVED_INPUT_FILE))
}
