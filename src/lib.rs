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

const THOUSAND: u64 = 1_000;
const MILLION: u64 = 1_000_000;

fn abbreviate(tokens: u64) -> String {
    let scaled = |unit: u64| tokens as f64 / unit as f64;

    if tokens < THOUSAND {
        tokens.to_string()
    } else if tokens < MILLION {
        format!("{:.1}K", scaled(THOUSAND))
    } else {
        format!("{:.1}M", scaled(MILLION))
    }
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

#[cfg(test)]
mod abbreviate_tests {
    use super::abbreviate;

    #[test]
    fn shows_counts_below_a_thousand_as_plain_digits() {
        assert_eq!(abbreviate(0), "0");
        assert_eq!(abbreviate(500), "500");
        assert_eq!(abbreviate(999), "999");
    }

    #[test]
    fn shows_thousands_with_one_decimal_place() {
        assert_eq!(abbreviate(1_000), "1.0K");
        assert_eq!(abbreviate(20_812), "20.8K");
        assert_eq!(abbreviate(999_999), "1000.0K");
    }

    #[test]
    fn shows_millions_with_one_decimal_place() {
        assert_eq!(abbreviate(1_000_000), "1.0M");
        assert_eq!(abbreviate(1_500_000), "1.5M");
    }

    #[test]
    fn rounds_to_the_nearest_decimal_place() {
        assert_eq!(abbreviate(20_850), "20.9K");
        assert_eq!(abbreviate(20_999), "21.0K");
    }
}
