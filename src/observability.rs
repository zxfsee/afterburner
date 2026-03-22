use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const OBS_JSONL_SINK_ENV: &str = "AFTERBURNER_OBS_JSONL_PATH";
pub const TRACEPARENT_ENV: &str = "AFTERBURNER_TRACEPARENT";
pub const TRACEPARENT_HEADER: &str = "traceparent";

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    pub traceparent: String,
    pub trace_id: String,
}

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

pub fn current_trace_context(namespace: &str) -> TraceContext {
    trace_context_from_env().unwrap_or_else(|| generate_trace_context(namespace))
}

pub fn trace_context_from_env() -> Option<TraceContext> {
    std::env::var(TRACEPARENT_ENV)
        .ok()
        .and_then(|value| parse_traceparent(value.as_str()))
}

pub fn trace_context_from_traceparent(raw: &str) -> Option<TraceContext> {
    parse_traceparent(raw)
}

pub fn trace_context_from_optional_traceparent(raw: Option<&str>, namespace: &str) -> TraceContext {
    raw.and_then(parse_traceparent)
        .unwrap_or_else(|| generate_trace_context(namespace))
}

pub fn trace_fields(trace: &TraceContext) -> Value {
    json!({
        "traceparent": trace.traceparent,
        "trace_id": trace.trace_id,
    })
}

pub fn attach_trace_fields(fields: &mut serde_json::Map<String, Value>, trace: &TraceContext) {
    fields.insert(
        "traceparent".to_string(),
        Value::from(trace.traceparent.clone()),
    );
    fields.insert("trace_id".to_string(), Value::from(trace.trace_id.clone()));
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

fn parse_traceparent(raw: &str) -> Option<TraceContext> {
    let raw = raw.trim().to_ascii_lowercase();
    let parts = raw.split('-').collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }
    let [version, trace_id, span_id, flags] = parts.as_slice() else {
        return None;
    };
    if version.len() != 2
        || trace_id.len() != 32
        || span_id.len() != 16
        || flags.len() != 2
        || !is_hex(version)
        || !is_hex(trace_id)
        || !is_hex(span_id)
        || !is_hex(flags)
        || is_all_zero(trace_id)
        || is_all_zero(span_id)
    {
        return None;
    }
    let trace_id = (*trace_id).to_string();
    Some(TraceContext {
        traceparent: raw.clone(),
        trace_id,
    })
}

fn generate_trace_context(namespace: &str) -> TraceContext {
    let counter = TRACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seed = format!("{namespace}:{now_ns}:{counter}:{}", std::process::id());
    let digest = Sha256::digest(seed.as_bytes());
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let trace_id = hex[0..32].to_string();
    let span_id = hex[32..48].to_string();
    let traceparent = format!("00-{trace_id}-{span_id}-01");
    TraceContext {
        traceparent,
        trace_id,
    }
}

fn is_hex(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_all_zero(value: &str) -> bool {
    value.bytes().all(|byte| byte == b'0')
}

#[cfg(test)]
mod tests {
    use super::{
        TraceContext, append_sink_line, event_line, trace_context_from_traceparent, trace_fields,
    };
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

    #[test]
    fn parse_traceparent_accepts_valid_w3c_header() {
        let trace = trace_context_from_traceparent(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        )
        .expect("valid traceparent");
        assert_eq!(
            trace,
            TraceContext {
                traceparent: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string(),
                trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            }
        );
    }

    #[test]
    fn trace_fields_include_traceparent_and_trace_id() {
        let fields = trace_fields(&TraceContext {
            traceparent: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string(),
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
        });
        assert_eq!(
            fields,
            json!({
                "traceparent": "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
                "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
            })
        );
    }
}
