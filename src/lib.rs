use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod args;
pub mod git;
pub mod input_log;

pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
}

#[derive(Deserialize)]
pub struct Session {
    #[serde(default, deserialize_with = "lenient")]
    model: Option<Model>,
    #[serde(default, deserialize_with = "lenient")]
    workspace: Option<Workspace>,
    #[serde(default, deserialize_with = "lenient")]
    context_window: Option<ContextWindow>,
    #[serde(default, deserialize_with = "lenient")]
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
    #[serde(default, deserialize_with = "lenient_number")]
    total_input_tokens: Option<u64>,
    #[serde(default, deserialize_with = "lenient_number")]
    remaining_percentage: Option<u8>,
}

#[derive(Deserialize)]
struct RateLimits {
    five_hour: Option<RateLimit>,
    seven_day: Option<RateLimit>,
}

#[derive(Deserialize)]
struct RateLimit {
    #[serde(default, deserialize_with = "lenient_number")]
    used_percentage: Option<u8>,
    #[serde(default, deserialize_with = "lenient_number")]
    resets_at: Option<u64>,
}

fn lenient<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;

    Ok(serde_json::from_value(value).ok())
}

fn lenient_number<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<u64>,
{
    let value = Value::deserialize(deserializer)?;

    Ok(whole_number(&value).and_then(|number| T::try_from(number).ok()))
}

fn whole_number(value: &Value) -> Option<u64> {
    value
        .as_f64()
        .map(f64::round)
        .filter(|number| *number >= 0.0 && *number < u64::MAX as f64)
        .map(|number| number as u64)
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

    fn five_hour(&self) -> Option<&RateLimit> {
        self.rate_limits.as_ref()?.five_hour.as_ref()
    }

    fn seven_day(&self) -> Option<&RateLimit> {
        self.rate_limits.as_ref()?.seven_day.as_ref()
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

/// Renders the line for a session parsed with [`Session::parse`]. `None`, a
/// session that did not parse, renders the warning line.
pub fn render(
    session: Option<&Session>,
    git_status: Option<&GitStatus>,
    now: SystemTime,
) -> String {
    let segments = session
        .map(|session| session_segments(session, git_status, now))
        .filter(|segments| !segments.is_empty())
        .unwrap_or_else(warning_segments);

    let segments: Vec<Segment> = std::iter::once(marker_segment()).chain(segments).collect();

    render_line(&segments) + "\n"
}

fn marker_segment() -> Segment {
    segment(202, 231, "🦀".to_string())
}

fn session_segments(
    session: &Session,
    git_status: Option<&GitStatus>,
    now: SystemTime,
) -> Vec<Segment> {
    [
        directory_segments(session.current_directory(), git_status),
        model_segments(session),
        context_segments(session),
        rate_limit_segments(session, now),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn warning_segments() -> Vec<Segment> {
    vec![segment(238, 231, "⚠ statusline".to_string())]
}

fn directory_segments(current_dir: Option<&str>, git_status: Option<&GitStatus>) -> Vec<Segment> {
    let directory = current_dir
        .and_then(directory_name)
        .map(|name| segment(136, 231, name));
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

fn rate_limit_segments(session: &Session, now: SystemTime) -> Vec<Segment> {
    let now = seconds_since_epoch(now);
    let five_hour = session
        .five_hour()
        .and_then(|limit| rate_limit_text('◷', limit, now))
        .map(|text| segment(60, 231, text));
    let seven_day = session
        .seven_day()
        .and_then(|limit| rate_limit_text('▦', limit, now))
        .map(|text| segment(66, 231, text));

    [five_hour, seven_day].into_iter().flatten().collect()
}

fn rate_limit_text(symbol: char, limit: &RateLimit, now: u64) -> Option<String> {
    let used = limit.used_percentage?;
    let countdown = limit
        .resets_at
        .and_then(|resets_at| resets_at.checked_sub(now))
        .filter(|remaining| *remaining > 0)
        .map(|remaining| format!(" {}", countdown(remaining)))
        .unwrap_or_default();

    Some(format!("{} {}%{}", symbol, used, countdown))
}

fn seconds_since_epoch(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

const SECONDS_PER_MINUTE: u64 = 60;
const MINUTES_PER_HOUR: u64 = 60;
const MINUTES_PER_DAY: u64 = 24 * MINUTES_PER_HOUR;

/// The two largest non-zero units of a wait, rounded up to the minute so a
/// reset still in the future never reads `0m`.
fn countdown(seconds: u64) -> String {
    let minutes = seconds.div_ceil(SECONDS_PER_MINUTE);
    let days = minutes / MINUTES_PER_DAY;
    let hours = minutes % MINUTES_PER_DAY / MINUTES_PER_HOUR;
    let minutes = minutes % MINUTES_PER_HOUR;

    let units = if days > 0 {
        [(days, 'd'), (hours, 'h')]
    } else if hours > 0 {
        [(hours, 'h'), (minutes, 'm')]
    } else {
        [(minutes, 'm'), (0, 'm')]
    };

    units
        .iter()
        .filter(|(amount, _)| *amount > 0)
        .map(|(amount, unit)| format!("{}{}", amount, unit))
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

/// The last non-empty component, so `/x/project/` still names `project`.
/// `/` and `""` have no name, and the segment is dropped rather than drawn
/// empty.
fn directory_name(current_dir: &str) -> Option<String> {
    current_dir
        .rsplit('/')
        .find(|component| !component.is_empty())
        .map(str::to_string)
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
mod countdown_tests {
    use super::countdown;

    #[test]
    fn shows_minutes_under_an_hour() {
        assert_eq!(countdown(45 * 60), "45m");
    }

    #[test]
    fn rounds_a_part_minute_up_so_it_never_reads_zero() {
        assert_eq!(countdown(1), "1m");
        assert_eq!(countdown(14 * 60 + 1), "15m");
    }

    #[test]
    fn shows_hours_and_minutes_under_a_day() {
        assert_eq!(countdown(2 * 3600 + 14 * 60), "2h14m");
    }

    #[test]
    fn shows_days_and_hours_from_a_day_up() {
        assert_eq!(countdown(3 * 86400 + 4 * 3600 + 59 * 60), "3d4h");
    }

    #[test]
    fn drops_a_second_unit_that_is_zero() {
        assert_eq!(countdown(2 * 3600), "2h");
        assert_eq!(countdown(3 * 86400), "3d");
    }

    #[test]
    fn carries_rounding_into_the_larger_unit() {
        assert_eq!(countdown(3600 - 1), "1h");
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
