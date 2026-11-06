use claude_statusline_rust::input_log::{append, record_line};
use std::fs;
use std::path::{Path, PathBuf};

struct TemporaryLog {
    path: PathBuf,
}

impl TemporaryLog {
    fn new(name: &str) -> Self {
        let directory = std::env::temp_dir().join(format!("statusline-log-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        TemporaryLog {
            path: directory.join("input.jsonl"),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn lines(&self) -> Vec<String> {
        fs::read_to_string(&self.path)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn contents(&self) -> String {
        fs::read_to_string(&self.path).unwrap()
    }
}

impl Drop for TemporaryLog {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.path.parent().unwrap());
    }
}

#[test]
fn keeps_compact_json_as_it_arrived() {
    let session_json = r#"{"model":{"display_name":"Opus 5"}}"#;

    let record = record_line(session_json).unwrap();

    assert_eq!(record, session_json);
}

#[test]
fn collapses_pretty_printed_json_onto_one_line() {
    let session_json = "{\n  \"model\": {\n    \"display_name\": \"Opus 5\"\n  }\n}";

    let record = record_line(session_json).unwrap();

    assert!(!record.contains('\n'));
    assert_eq!(record, r#"{"model":{"display_name":"Opus 5"}}"#);
}

#[test]
fn preserves_the_session_when_collapsing_it() {
    let session_json = include_str!("../initial-test-data/input.json");

    let record = record_line(session_json).unwrap();

    let original: serde_json::Value = serde_json::from_str(session_json).unwrap();
    let saved: serde_json::Value = serde_json::from_str(&record).unwrap();
    assert_eq!(saved, original);
}

#[test]
fn saves_input_that_is_not_json_with_its_newlines_stripped() {
    let record = record_line("this is\nnot json\r\nat all").unwrap();

    assert!(!record.contains('\n'));
    assert!(!record.contains('\r'));
    assert!(record.contains("not json"));
}

#[test]
fn saves_truncated_json_rather_than_discarding_it() {
    let session_json = include_str!("../initial-test-data/input.json");
    let truncated = &session_json[..session_json.len() / 2];

    let record = record_line(truncated).unwrap();

    assert!(!record.contains('\n'));
    assert!(!record.is_empty());
}

#[test]
fn has_no_record_for_empty_input() {
    assert!(record_line("").is_none());
}

#[test]
fn has_no_record_for_whitespace_only_input() {
    assert!(record_line("  \n\t\r\n ").is_none());
}

#[test]
fn creates_the_log_when_it_does_not_exist() {
    let log = TemporaryLog::new("creates");

    append(log.path(), r#"{"a":1}"#);

    assert_eq!(log.lines(), vec![r#"{"a":1}"#]);
}

#[test]
fn appends_each_session_in_the_order_it_arrived() {
    let log = TemporaryLog::new("appends");

    append(log.path(), r#"{"a":1}"#);
    append(log.path(), r#"{"b":2}"#);

    assert_eq!(log.lines(), vec![r#"{"a":1}"#, r#"{"b":2}"#]);
}

#[test]
fn ends_each_record_with_a_single_newline() {
    let log = TemporaryLog::new("newline");

    append(log.path(), r#"{"a":1}"#);
    append(log.path(), r#"{"b":2}"#);

    assert_eq!(log.contents(), "{\"a\":1}\n{\"b\":2}\n");
}

#[test]
fn writes_pretty_printed_sessions_as_one_line_each() {
    let log = TemporaryLog::new("pretty");

    append(log.path(), "{\n  \"a\": 1\n}");
    append(log.path(), "{\n  \"b\": 2\n}");

    assert_eq!(log.lines(), vec![r#"{"a":1}"#, r#"{"b":2}"#]);
}

#[test]
fn writes_nothing_for_empty_input() {
    let log = TemporaryLog::new("empty");

    append(log.path(), "");

    assert!(!log.exists());
}

#[test]
fn does_not_panic_when_the_log_cannot_be_written() {
    let unwritable = Path::new("/does/not/exist/input.jsonl");

    append(unwritable, r#"{"a":1}"#);
}

#[test]
fn keeps_every_record_whole_when_sessions_append_at_once() {
    const WRITERS: usize = 8;
    const RECORDS_PER_WRITER: usize = 400;

    let log = TemporaryLog::new("concurrent");

    std::thread::scope(|scope| {
        for writer in 0..WRITERS {
            let path = log.path();
            scope.spawn(move || {
                for record in 0..RECORDS_PER_WRITER {
                    let session_json = format!(r#"{{"writer":{},"record":{}}}"#, writer, record);
                    append(path, &session_json);
                }
            });
        }
    });

    let lines = log.lines();
    let whole = lines
        .iter()
        .filter(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
        .count();

    assert_eq!(lines.len(), WRITERS * RECORDS_PER_WRITER);
    assert_eq!(whole, lines.len(), "some records were split");
}
