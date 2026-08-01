use serde::Deserialize;

pub mod git;

pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
}

#[derive(Deserialize)]
pub struct Session {
    model: Option<Model>,
    workspace: Option<Workspace>,
    context_window: Option<ContextWindow>,
}

#[derive(Deserialize)]
struct Model {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: Option<String>,
}

#[derive(Deserialize)]
struct ContextWindow {
    total_input_tokens: Option<u64>,
    remaining_percentage: Option<u8>,
}

impl Session {
    pub fn parse(session_json: &str) -> Option<Session> {
        serde_json::from_str(session_json).ok()
    }

    pub fn current_directory(&self) -> Option<&str> {
        self.workspace.as_ref()?.current_dir.as_deref()
    }

    fn model_name(&self) -> Option<&str> {
        self.model.as_ref()?.display_name.as_deref()
    }

    fn total_input_tokens(&self) -> Option<u64> {
        self.context_window.as_ref()?.total_input_tokens
    }

    fn remaining_percentage(&self) -> Option<u8> {
        self.context_window.as_ref()?.remaining_percentage
    }
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

pub fn render(session_json: &str, git_status: Option<&GitStatus>) -> String {
    let statusline = match Session::parse(session_json) {
        Some(session) => render_session(&session, git_status),
        None => String::new(),
    };

    if statusline.is_empty() {
        return render_lines(&[vec![segment(238, 231, "⚠ statusline".to_string())]]);
    }

    statusline
}

fn render_session(session: &Session, git_status: Option<&GitStatus>) -> String {
    let lines = [
        directory_line(session.current_directory(), git_status),
        model_line(session),
        context_line(session),
    ];

    render_lines(&lines)
}

fn render_lines(lines: &[Vec<Segment>]) -> String {
    lines
        .iter()
        .filter(|segments| !segments.is_empty())
        .map(|segments| render_line(segments) + "\n")
        .collect()
}

fn directory_line(current_dir: Option<&str>, git_status: Option<&GitStatus>) -> Vec<Segment> {
    let directory = current_dir.map(|dir| segment(136, 231, directory_name(dir)));
    let git = git_status.map(|status| segment(238, 231, git_description(status)));

    [directory, git].into_iter().flatten().collect()
}

fn model_line(session: &Session) -> Vec<Segment> {
    let model = session
        .model_name()
        .map(|name| segment(237, 231, format!("✱ {}", name)));

    let tokens = session
        .total_input_tokens()
        .map(|tokens| segment(235, 51, format!("§ {} tokens", abbreviate(tokens))));

    [model, tokens].into_iter().flatten().collect()
}

fn context_line(session: &Session) -> Vec<Segment> {
    match (session.total_input_tokens(), session.remaining_percentage()) {
        (Some(tokens), Some(remaining)) => vec![segment(
            240,
            251,
            format!("◔ {} ({}%)", group_digits(tokens), remaining),
        )],
        _ => Vec::new(),
    }
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

const DIGITS_PER_GROUP: usize = 3;

fn group_digits(tokens: u64) -> String {
    let digits: Vec<char> = tokens.to_string().chars().collect();

    let groups: Vec<String> = digits
        .rchunks(DIGITS_PER_GROUP)
        .rev()
        .map(|group| group.iter().collect())
        .collect();

    groups.join(",")
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

#[cfg(test)]
mod group_digits_tests {
    use super::group_digits;

    #[test]
    fn leaves_counts_of_three_digits_or_fewer_ungrouped() {
        assert_eq!(group_digits(0), "0");
        assert_eq!(group_digits(7), "7");
        assert_eq!(group_digits(999), "999");
    }

    #[test]
    fn separates_thousands_with_a_comma() {
        assert_eq!(group_digits(1_000), "1,000");
        assert_eq!(group_digits(20_812), "20,812");
        assert_eq!(group_digits(999_999), "999,999");
    }

    #[test]
    fn separates_each_group_of_three_digits() {
        assert_eq!(group_digits(1_000_000), "1,000,000");
        assert_eq!(group_digits(20_812_000), "20,812,000");
    }
}
