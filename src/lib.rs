use serde::Deserialize;

pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
}

#[derive(Deserialize)]
struct Session {
    model: Model,
    workspace: Workspace,
    context_window: ContextWindow,
}

#[derive(Deserialize)]
struct Model {
    display_name: String,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: String,
}

#[derive(Deserialize)]
struct ContextWindow {
    total_input_tokens: u64,
    remaining_percentage: u8,
}

struct Segment {
    background: u8,
    foreground: u8,
    text: String,
}

fn segment(background: u8, foreground: u8, text: String) -> Segment {
    Segment {
        background,
        foreground,
        text,
    }
}

const RESET: &str = "\x1b[0m";
const SEPARATOR: char = '\u{e0b0}';

pub fn render(session_json: &str, git_status: &GitStatus) -> String {
    let session: Session = serde_json::from_str(session_json).unwrap();

    let context_window = &session.context_window;

    let lines = [
        vec![
            segment(136, 231, directory_name(&session.workspace.current_dir)),
            segment(238, 231, git_description(git_status)),
        ],
        vec![
            segment(237, 231, format!("✱ {}", session.model.display_name)),
            segment(
                235,
                51,
                format!(
                    "§ {} tokens",
                    abbreviate(context_window.total_input_tokens)
                ),
            ),
        ],
        vec![segment(
            240,
            251,
            format!(
                "◔ {} ({}%)",
                group_digits(context_window.total_input_tokens),
                context_window.remaining_percentage
            ),
        )],
    ];

    lines
        .iter()
        .map(|segments| render_line(segments) + "\n")
        .collect()
}

fn render_line(segments: &[Segment]) -> String {
    let mut line = String::from(RESET);

    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            let previous_background = segments[index - 1].background;
            line += RESET;
            line += &background(segment.background);
            line += &foreground(previous_background);
            line.push(SEPARATOR);
        }
        line += &background(segment.background);
        line += &foreground(segment.foreground);
        line += &format!(" {} ", segment.text);
    }

    let last_background = segments[segments.len() - 1].background;
    line += RESET;
    line += &foreground(last_background);
    line.push(SEPARATOR);
    line += RESET;

    line
}

fn background(colour: u8) -> String {
    format!("\x1b[48;5;{}m", colour)
}

fn foreground(colour: u8) -> String {
    format!("\x1b[38;5;{}m", colour)
}

fn directory_name(current_dir: &str) -> String {
    current_dir.rsplit('/').next().unwrap_or(current_dir).to_string()
}

fn git_description(git_status: &GitStatus) -> String {
    let dirty_marker = if git_status.dirty { " ●" } else { "" };
    format!("⎇ {}{}", git_status.branch, dirty_marker)
}

fn abbreviate(tokens: u64) -> String {
    format!("{:.1}K", tokens as f64 / 1000.0)
}

fn group_digits(tokens: u64) -> String {
    let digits = tokens.to_string();
    let mut grouped = String::new();

    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }

    grouped
}
