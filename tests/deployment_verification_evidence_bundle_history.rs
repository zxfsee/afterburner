use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::fixture_path;

#[test]
fn deployment_verification_bundle_history_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_history.schema.json",
    ))
    .expect("read deployment verification bundle history schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification bundle history schema");

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
        "bundle_path",
        "receipt_path",
        "artifact_version",
        "profile_name",
        "verification_status",
        "evidence_count",
        "evidence_sources",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn deployment_verification_bundle_history_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp
        .path()
        .join("deployment_verification_evidence_bundle.json");
    fs::write(
        &bundle,
        r#"{
  "schema_version": "1",
  "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed",
  "evidence_count": 2,
  "evidence_sources": [
    {"artifact_path":"artifacts/eval/mnist_eval_summary.json","event_name":"eval_done","observed_at_unix_ms":1735689601000},
    {"artifact_path":"artifacts/deploy/candidate_upload_request.json","event_name":"artifact_upload_request_written","observed_at_unix_ms":1735689602000}
  ]
}"#,
    )
    .expect("write bundle");
    let out = tmp
        .path()
        .join("deployment_verification_evidence_bundle_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("verification-bundle")
        .arg("record-history")
        .arg("--bundle")
        .arg(&bundle)
        .arg("--event")
        .arg("packaged")
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
    entry.insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "packaged",
                    "bundle_path": "<bundle>",
                    "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
                    "artifact_version": "0.2.0",
                    "profile_name": "staging-a",
                    "verification_status": "passed",
                    "evidence_count": 2,
                    "evidence_sources": [
                        {
                            "artifact_path": "artifacts/eval/mnist_eval_summary.json",
                            "event_name": "eval_done",
                            "observed_at_unix_ms": 1735689601000_u64
                        },
                        {
                            "artifact_path": "artifacts/deploy/candidate_upload_request.json",
                            "event_name": "artifact_upload_request_written",
                            "observed_at_unix_ms": 1735689602000_u64
                        }
                    ],
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
                == Some("deployment_verification_evidence_bundle_history_written")
        })
        .unwrap_or_else(|| {
            panic!("missing deployment verification bundle history event in stderr: {stderr}")
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
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}

#[test]
fn deployment_verification_bundle_history_grouped_verify_surface_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp
        .path()
        .join("deployment_verification_evidence_bundle.json");
    fs::write(
        &bundle,
        r#"{
  "schema_version": "1",
  "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed",
  "evidence_count": 2,
  "evidence_sources": [
    {"artifact_path":"artifacts/eval/mnist_eval_summary.json","event_name":"eval_done","observed_at_unix_ms":1735689601000},
    {"artifact_path":"artifacts/deploy/candidate_upload_request.json","event_name":"artifact_upload_request_written","observed_at_unix_ms":1735689602000}
  ]
}"#,
    )
    .expect("write bundle");
    let out = tmp
        .path()
        .join("deployment_verification_evidence_bundle_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("verify")
        .arg("bundle")
        .arg("history")
        .arg("record")
        .arg("--bundle")
        .arg(&bundle)
        .arg("--event")
        .arg("packaged")
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
    entry.insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "packaged",
                    "bundle_path": "<bundle>",
                    "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
                    "artifact_version": "0.2.0",
                    "profile_name": "staging-a",
                    "verification_status": "passed",
                    "evidence_count": 2,
                    "evidence_sources": [
                        {
                            "artifact_path": "artifacts/eval/mnist_eval_summary.json",
                            "event_name": "eval_done",
                            "observed_at_unix_ms": 1735689601000_u64
                        },
                        {
                            "artifact_path": "artifacts/deploy/candidate_upload_request.json",
                            "event_name": "artifact_upload_request_written",
                            "observed_at_unix_ms": 1735689602000_u64
                        }
                    ],
                    "recorded_at_unix_ms": 1735689800000_u64
                }
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|value| {
            value.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_history_written")
        })
        .unwrap_or_else(|| {
            panic!("missing deployment verification bundle history event in stderr: {stderr}")
        });
    let mut event = event;
    event
        .as_object_mut()
        .expect("event must be object")
        .insert("ts_ms".to_string(), Value::from(0));
    assert_eq!(
        event,
        serde_json::json!({
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_history_written",
            "ts_ms": 0,
            "fields": {
                "history_path": out.display().to_string(),
                "history": serde_json::json!({
                    "schema_version": "1",
                    "entries": [
                        {
                            "event": "packaged",
                            "bundle_path": bundle.display().to_string(),
                            "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
                            "artifact_version": "0.2.0",
                            "profile_name": "staging-a",
                            "verification_status": "passed",
                            "evidence_count": 2,
                            "evidence_sources": [
                                {
                                    "artifact_path": "artifacts/eval/mnist_eval_summary.json",
                                    "event_name": "eval_done",
                                    "observed_at_unix_ms": 1735689601000_u64
                                },
                                {
                                    "artifact_path": "artifacts/deploy/candidate_upload_request.json",
                                    "event_name": "artifact_upload_request_written",
                                    "observed_at_unix_ms": 1735689602000_u64
                                }
                            ],
                            "recorded_at_unix_ms": 1735689800000_u64
                        }
                    ]
                })
            }
        })
    );
}
