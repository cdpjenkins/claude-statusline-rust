use serde::Deserialize;

pub mod args;
pub mod git;
pub mod input_log;

pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
}

#[derive(Deserialize)]
pub struct Session {
    model: Option<Model>,
    workspace: Option<Workspace>,
    context_window: Option<ContextWindow>,
    rate_limits: Option<RateLimits>,
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

#[derive(Deserialize)]
struct RateLimits {
    five_hour: Option<RateLimit>,
    seven_day: Option<RateLimit>,
}

#[derive(Deserialize)]
struct RateLimit {
    used_percentage: Option<u8>,
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

    fn five_hour_used(&self) -> Option<u8> {
        self.rate_limits.as_ref()?.five_hour.as_ref()?.used_percentage
    }

    fn seven_day_used(&self) -> Option<u8> {
        self.rate_limits.as_ref()?.seven_day.as_ref()?.used_percentage
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
    let segments = Session::parse(session_json)
        .map(|session| session_segments(&session, git_status))
        .filter(|segments| !segments.is_empty())
        .unwrap_or_else(warning_segments);

    render_line(&segments) + "\n"
}

fn session_segments(session: &Session, git_status: Option<&GitStatus>) -> Vec<Segment> {
    [
        directory_segments(session.current_directory(), git_status),
        model_segments(session),
        context_segments(session),
        rate_limit_segments(session),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn warning_segments() -> Vec<Segment> {
    vec![segment(238, 231, "⚠ statusline".to_string())]
}

fn directory_segments(current_dir: Option<&str>, git_status: Option<&GitStatus>) -> Vec<Segment> {
    let directory = current_dir.map(|dir| segment(136, 231, directory_name(dir)));
    let git = git_status.map(|status| segment(238, 231, git_description(status)));

    [directory, git].into_iter().flatten().collect()
}

fn model_segments(session: &Session) -> Vec<Segment> {
    session
        .model_name()
        .map(|name| segment(237, 231, format!("✱ {}", name)))
        .into_iter()
        .collect()
}

fn context_segments(session: &Session) -> Vec<Segment> {
    match (session.total_input_tokens(), session.remaining_percentage()) {
        (Some(tokens), Some(remaining)) => vec![segment(
            240,
            251,
            format!("◔ {} ({}%)", group_digits(tokens), remaining),
        )],
        _ => Vec::new(),
    }
}

fn rate_limit_segments(session: &Session) -> Vec<Segment> {
    let five_hour = session
        .five_hour_used()
        .map(|used| segment(60, 231, format!("◷ {}%", used)));
    let seven_day = session
        .seven_day_used()
        .map(|used| segment(66, 231, format!("▦ {}%", used)));

    [five_hour, seven_day].into_iter().flatten().collect()
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
