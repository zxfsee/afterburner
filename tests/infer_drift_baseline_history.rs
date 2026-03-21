use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn infer_output_drift_baseline_history_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_history.schema.json",
    ))
    .expect("read infer drift baseline history schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline history");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-history/v1")
    );
    let entries = schema
        .get("properties")
        .and_then(|value| value.get("entries"))
        .and_then(|value| value.get("items"))
        .and_then(Value::as_object)
        .expect("entries.items must be an object");
    let required = entries
        .get("required")
        .and_then(Value::as_array)
        .expect("entries.items.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "event",
        "pointer_path",
        "approval_path",
        "artifact",
        "artifact_version",
        "policy_profile",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile
            .contains("drift-record-approved-baseline-history pointer event recorded_at_unix_ms:"),
        "justfile must expose the drift-record-approved-baseline-history workflow"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("infer_output_drift_baseline_history.json"),
        "README must mention the baseline history artifact"
    );
    assert!(
        readme.contains("just drift-record-approved-baseline-history"),
        "README must mention the baseline history workflow"
    );
}

#[test]
fn drift_record_approved_baseline_history_appends_entry_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pointer = tmp.path().join("pointer.json");
    fs::write(
        &pointer,
        r#"{"schema_version":"1","approval_path":"approval.json","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default"}"#,
    )
    .expect("write pointer");
    let out = tmp.path().join("history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("record-approved-baseline-history")
        .arg("--pointer")
        .arg(&pointer)
        .arg("--event")
        .arg("pointed")
        .arg("--recorded-at-unix-ms")
        .arg("1735689800000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let history_text = fs::read_to_string(&out).expect("read history");
    let mut history: Value = serde_json::from_str(&history_text).expect("parse history");
    let entries = history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("history entries must be array");
    let entry = entries[0].as_object_mut().expect("entry must be object");
    entry.insert("pointer_path".to_string(), Value::from("<pointer>"));
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "pointed",
                    "pointer_path": "<pointer>",
                    "approval_path": "approval.json",
                    "artifact": "artifacts/inference/0.2.0/model.mpk",
                    "artifact_version": "0.2.0",
                    "policy_profile": "promotion-default",
                    "recorded_at_unix_ms": 1735689800000_u64
                }
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_history_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("history_path".to_string(), Value::from("<history>"));
    let history_obj = fields
        .get_mut("history")
        .and_then(Value::as_object_mut)
        .expect("event history must be object");
    let entries = history_obj
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("event history entries must be array");
    entries[0]
        .as_object_mut()
        .expect("event history entry must be object")
        .insert("pointer_path".to_string(), Value::from("<pointer>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}
