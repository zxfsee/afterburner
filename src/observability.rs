use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

pub const OBS_JSONL_SINK_ENV: &str = "AFTERBURNER_OBS_JSONL_PATH";

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

pub fn append_json_line(path: &Path, line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(&mut file, "{line}")?;
    Ok(())
}

pub fn event_line(level: &str, source: &str, event: &str, fields: Value) -> String {
    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    json!({
        "ts_ms": ts_ms,
        "level": level,
        "source": source,
        "event": event,
        "fields": fields
    })
    .to_string()
}

pub fn emit_event(level: &str, source: &str, event: &str, fields: Value) {
    let line = event_line(level, source, event, fields);
    eprintln!("{line}");

    if let Err(err) = append_sink_line_from_env(&line) {
        let detail = json_escape(&err.to_string());
        let sink_path = std::env::var(OBS_JSONL_SINK_ENV).unwrap_or_default();
        let sink_path = json_escape(&sink_path);
        let fallback = format!(
            r#"{{"level":"error","source":"observability","event":"sink_write_failed","fields":{{"env":"{OBS_JSONL_SINK_ENV}","path":"{sink_path}","detail":"{detail}"}}}}"#
        );
        eprintln!("{fallback}");
    }
}

pub fn append_sink_line(path: Option<&Path>, line: &str) -> io::Result<()> {
    if let Some(path) = path {
        append_json_line(path, line)?;
    }
    Ok(())
}

fn append_sink_line_from_env(line: &str) -> io::Result<()> {
    let sink_path = sink_path_from_env();
    append_sink_line(sink_path.as_deref(), line)
}

fn sink_path_from_env() -> Option<PathBuf> {
    std::env::var(OBS_JSONL_SINK_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{append_sink_line, event_line};
    use serde_json::json;

    #[test]
    fn event_line_uses_normalized_envelope() {
        let line = event_line("info", "infer_cli", "infer_done", json!({"elapsed_ms": 1}));
        assert!(line.contains(r#""level":"info""#));
        assert!(line.contains(r#""source":"infer_cli""#));
        assert!(line.contains(r#""event":"infer_done""#));
        assert!(line.contains(r#""fields":{"elapsed_ms":1}"#));
    }

    #[test]
    fn append_sink_line_is_optional() {
        append_sink_line(None, r#"{"ok":true}"#).expect("none sink should be no-op");
    }
}
