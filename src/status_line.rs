use serde::Deserialize;

use crate::git::GitInfo;

const ARROW: char = '\u{e0b0}';

#[derive(Deserialize)]
struct Session {
    model: Model,
    workspace: Workspace,
    cost: Cost,
    context_window: ContextWindow,
    rate_limits: RateLimits,
}

#[derive(Deserialize)]
struct Model {
    display_name: String,
}

#[derive(Deserialize)]
struct Workspace {
    project_dir: String,
}

#[derive(Deserialize)]
struct Cost {
    total_cost_usd: f64,
}

#[derive(Deserialize)]
struct ContextWindow {
    total_input_tokens: u64,
    remaining_percentage: Option<f64>,
}

#[derive(Deserialize)]
struct RateLimits {
    five_hour: RateLimitWindow,
}

#[derive(Deserialize)]
struct RateLimitWindow {
    used_percentage: f64,
}

struct Segment {
    background: u8,
    foreground: u8,
    content: String,
}

pub fn render(input: &str, git: &dyn GitInfo) -> String {
    let session: Session = serde_json::from_str(input).expect("valid session json");

    let segments = segments_for(&session, git);

    let mut line = String::new();
    let mut previous_background: Option<u8> = None;

    for segment in &segments {
        line.push_str("\x1b[0m");
        line.push_str(&format!("\x1b[48;5;{}m", segment.background));
        if let Some(previous) = previous_background {
            line.push_str(&format!("\x1b[38;5;{}m", previous));
            line.push(ARROW);
            line.push_str(&format!("\x1b[48;5;{}m", segment.background));
        }
        line.push_str(&format!("\x1b[38;5;{}m", segment.foreground));
        line.push_str(&format!(" {} ", segment.content));
        previous_background = Some(segment.background);
    }

    line.push_str("\x1b[0m");
    if let Some(previous) = previous_background {
        line.push_str(&format!("\x1b[38;5;{}m", previous));
        line.push(ARROW);
    }
    line.push_str("\x1b[0m");

    line
}

fn segments_for(session: &Session, git: &dyn GitInfo) -> Vec<Segment> {
    let mut segments = vec![Segment {
        background: 136,
        foreground: 231,
        content: project_name(&session.workspace.project_dir),
    }];

    if let Some(description) = git.describe() {
        segments.push(Segment {
            background: 238,
            foreground: 231,
            content: format!("⎇ {}", description),
        });
    }

    segments.push(Segment {
        background: 237,
        foreground: 231,
        content: format!("✱ {}", session.model.display_name),
    });

    segments.push(Segment {
        background: 235,
        foreground: 51,
        content: format!("§ {} tokens", session.context_window.total_input_tokens),
    });

    segments.push(Segment {
        background: 234,
        foreground: 157,
        content: format!(
            "☉ ${:.2} {}%",
            session.cost.total_cost_usd, session.rate_limits.five_hour.used_percentage
        ),
    });

    segments.push(Segment {
        background: 240,
        foreground: 251,
        content: format!(
            "◔ {} ({}%)",
            session.context_window.total_input_tokens,
            session.context_window.remaining_percentage.unwrap_or(100.0)
        ),
    });

    segments
}

fn project_name(project_dir: &str) -> String {
    project_dir
        .rsplit('/')
        .next()
        .unwrap_or(project_dir)
        .to_string()
}
