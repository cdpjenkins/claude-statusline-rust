use claude_statusline_rust::{git, render, Session};
use std::io::Read;
use std::path::Path;

fn main() {
    let mut session_json = String::new();
    let _ = std::io::stdin().read_to_string(&mut session_json);

    let current_dir = Session::parse(&session_json)
        .and_then(|session| session.current_directory().map(str::to_string));

    let git_status = current_dir.and_then(|dir| git::read(Path::new(&dir)));

    print!("{}", render(&session_json, git_status.as_ref()));
}
