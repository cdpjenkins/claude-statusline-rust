use std::io::{self, Read, Write};

use claude_statusline_rust::git::GitCommand;
use claude_statusline_rust::status_line::render;

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("read session json from stdin");

    let git = GitCommand {
        working_dir: ".".to_string(),
    };

    let line = render(&input, &git);

    io::stdout()
        .write_all(line.as_bytes())
        .expect("write status line to stdout");
}
