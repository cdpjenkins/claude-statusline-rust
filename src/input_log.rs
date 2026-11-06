use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn append(path: &Path, session_json: &str) {
    let Some(record) = record_line(session_json) else {
        return;
    };

    // One write_all, not writeln!: O_APPEND keeps a single write whole
    // against other sessions, but writeln! writes the record and its newline
    // separately, and another session's record can land between them.
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut log| log.write_all(format!("{}\n", record).as_bytes()));
}

pub fn record_line(session_json: &str) -> Option<String> {
    if session_json.trim().is_empty() {
        return None;
    }

    Some(compact(session_json).unwrap_or_else(|| strip_newlines(session_json)))
}

fn compact(session_json: &str) -> Option<String> {
    let session: Value = serde_json::from_str(session_json).ok()?;

    serde_json::to_string(&session).ok()
}

fn strip_newlines(session_json: &str) -> String {
    session_json
        .chars()
        .filter(|character| *character != '\n' && *character != '\r')
        .collect()
}
