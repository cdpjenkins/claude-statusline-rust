use claude_statusline_rust::{current_directory, git, render};
use std::io::Read;
use std::path::Path;

fn main() {
    let mut session_json = String::new();
    std::io::stdin().read_to_string(&mut session_json).unwrap();

    let current_dir = current_directory(&session_json);
    let git_status = git::read(Path::new(&current_dir));

    print!("{}", render(&session_json, git_status.as_ref()));
}
