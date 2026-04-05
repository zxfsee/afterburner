use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::fixture_path;

#[test]
fn deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history_schema_is_explicit()
 {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history.schema.json",
    ))
    .expect(
        "read deployment verification evidence bundle rollback supersession reconciliation history schema",
    );
    let schema: Value = serde_json::from_str(&schema_text).expect(
        "parse deployment verification evidence bundle rollback supersession reconciliation history schema",
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
        "reconciliation_path",
        "supersession_path",
        "current_supersession_path",
        "reconciliation_status",
        "desired",
        "current",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history_writes_history_and_event()
 {
    let tmp = tempfile::tempdir().expect("tempdir");
    let reconciliation = tmp
        .path()
        .join("deployment_verification_evidence_bundle_rollback_supersession_reconciliation.json");
    fs::write(
        &reconciliation,
        r#"{
  "schema_version": "1",
  "supersession_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.json",
  "current_supersession_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.current.json",
  "desired": {
    "previous_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.json",
    "next_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.next.json",
    "previous_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
    "next_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.older.json",
    "previous_restored_artifact_version": "0.1.0",
    "next_restored_artifact_version": "0.0.9",
    "profile_name": "staging-a",
    "superseded_at_unix_ms": 1735689900000
  },
  "current": {
    "current_supersession_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.current.json",
    "previous_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.json",
    "next_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.next.json",
    "previous_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
    "next_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.older.json",
    "previous_restored_artifact_version": "0.1.0",
    "next_restored_artifact_version": "0.0.9",
    "profile_name": "staging-a",
    "superseded_at_unix_ms": 1735689900000
  },
  "reconciliation_status": "aligned"
}"#,
    )
    .expect("write reconciliation");
    let out = tmp.path().join(
        "deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history.json",
    );

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("verification-bundle")
        .arg("record-rollback-supersession-reconciliation-history")
        .arg("--reconciliation")
        .arg(&reconciliation)
        .arg("--event")
        .arg("reconciled")
        .arg("--recorded-at-unix-ms")
        .arg("1735689950000")
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
    entry.insert(
        "reconciliation_path".to_string(),
        Value::from("<reconciliation>"),
    );
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "reconciled",
                    "reconciliation_path": "<reconciliation>",
                    "supersession_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.json",
                    "current_supersession_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.current.json",
                    "reconciliation_status": "aligned",
                    "desired": {
                        "previous_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.json",
                        "next_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.next.json",
                        "previous_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
                        "next_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.older.json",
                        "previous_restored_artifact_version": "0.1.0",
                        "next_restored_artifact_version": "0.0.9",
                        "profile_name": "staging-a",
                        "superseded_at_unix_ms": 1735689900000_u64
                    },
                    "current": {
                        "current_supersession_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.current.json",
                        "previous_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.json",
                        "next_rollback_path": "artifacts/deploy/deployment_verification_evidence_bundle_rollback.next.json",
                        "previous_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
                        "next_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.older.json",
                        "previous_restored_artifact_version": "0.1.0",
                        "next_restored_artifact_version": "0.0.9",
                        "profile_name": "staging-a",
                        "superseded_at_unix_ms": 1735689900000_u64
                    },
                    "recorded_at_unix_ms": 1735689950000_u64
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
                == Some(
                    "deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history_written",
                )
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification evidence bundle rollback supersession reconciliation history event in stderr: {stderr}"
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
        .insert(
            "reconciliation_path".to_string(),
            Value::from("<reconciliation>"),
        );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}
