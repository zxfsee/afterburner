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
fn deployment_verification_receipt_transport_locator_history_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_receipt_transport_locator_history.schema.json",
    ))
    .expect("read deployment verification receipt transport locator history schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification receipt transport locator history schema");

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
        "locator_path",
        "receipt_path",
        "artifact_version",
        "profile_name",
        "verification_status",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "deployment-verification-record-receipt-transport-locator-history locator event recorded_at_unix_ms:"
        ),
        "justfile must expose the deployment-verification-record-receipt-transport-locator-history workflow"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("deployment_verification_receipt_transport_locator_history.json"),
        "workflow reference must mention the deployment verification receipt transport locator history artifact"
    );
    assert!(
        workflows.contains("just deployment-verification-record-receipt-transport-locator-history"),
        "workflow reference must mention the deployment verification receipt transport locator history workflow"
    );
}

#[test]
fn deployment_verification_receipt_transport_locator_history_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let locator = tmp
        .path()
        .join("deployment_verification_receipt_transport_locator.json");
    fs::write(
        &locator,
        r#"{
  "schema_version": "1",
  "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed"
}"#,
    )
    .expect("write locator");
    let out = tmp
        .path()
        .join("deployment_verification_receipt_transport_locator_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("record-verification-receipt-transport-locator-history")
        .arg("--locator")
        .arg(&locator)
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
    entry.insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "pointed",
                    "locator_path": "<locator>",
                    "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
                    "artifact_version": "0.2.0",
                    "profile_name": "staging-a",
                    "verification_status": "passed",
                    "recorded_at_unix_ms": 1735689800000_u64
                }
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_receipt_transport_locator_history_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification receipt transport locator history event in stderr: {stderr}"
            )
        });
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
        .insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_receipt_transport_locator_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}
