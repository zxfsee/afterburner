use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn infer_output_drift_baseline_supersession_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_supersession.schema.json",
    ))
    .expect("read infer drift baseline supersession schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline supersession");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-supersession/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "previous_approval_path",
        "next_approval_path",
        "superseded_by_approval_ticket",
        "previous_artifact_version",
        "next_artifact_version",
        "policy_profile",
        "superseded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("infer_output_drift_baseline_supersession.json"),
        "workflow reference must mention the baseline supersession artifact"
    );
}

#[test]
fn drift_supersede_baseline_approval_writes_supersession_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let previous = tmp.path().join("previous.json");
    let next = tmp.path().join("next.json");
    fs::write(
        &previous,
        r#"{"schema_version":"1","baseline_path":"baseline-a.json","artifact":"artifacts/inference/0.1.0/model.mpk","artifact_version":"0.1.0","policy_profile":"promotion-default","approved_by":"ops-review","approval_ticket":"CHG-1000","approved_at_unix_ms":1735689500000}"#,
    )
    .expect("write previous approval");
    fs::write(
        &next,
        r#"{"schema_version":"1","baseline_path":"baseline-b.json","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default","approved_by":"ops-review","approval_ticket":"CHG-2000","approved_at_unix_ms":1735689600000}"#,
    )
    .expect("write next approval");

    let out = tmp.path().join("supersession.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("supersede-baseline-approval")
        .arg("--previous-approval")
        .arg(&previous)
        .arg("--next-approval")
        .arg(&next)
        .arg("--superseded-at-unix-ms")
        .arg("1735689700000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read supersession");
    let mut supersession: Value = serde_json::from_str(&text).expect("parse supersession");
    let object = supersession
        .as_object_mut()
        .expect("supersession must be a top-level object");
    object.insert(
        "previous_approval_path".to_string(),
        Value::from("<previous>"),
    );
    object.insert("next_approval_path".to_string(), Value::from("<next>"));
    assert_eq!(
        supersession,
        serde_json::json!({
            "schema_version": "1",
            "previous_approval_path": "<previous>",
            "next_approval_path": "<next>",
            "superseded_by_approval_ticket": "CHG-2000",
            "previous_artifact_version": "0.1.0",
            "next_artifact_version": "0.2.0",
            "policy_profile": "promotion-default",
            "superseded_at_unix_ms": 1735689700000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_approval_superseded");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("supersession_path".to_string(), Value::from("<out>"));
    let supersession_obj = fields
        .get_mut("supersession")
        .and_then(Value::as_object_mut)
        .expect("event supersession must be an object");
    supersession_obj.insert(
        "previous_approval_path".to_string(),
        Value::from("<previous>"),
    );
    supersession_obj.insert("next_approval_path".to_string(), Value::from("<next>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_approval_superseded",
            "fields": {
                "supersession_path": "<out>",
                "supersession": supersession
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
