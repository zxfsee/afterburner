use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn deployment_stack_launch_handoff_history_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_evidence_handoff_history.schema.json",
    ))
    .expect("read deployment stack launch handoff history schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment stack launch handoff history schema");

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
        "handoff_path",
        "bundle_path",
        "profile_name",
        "deploy_hostname",
        "deployable_unit",
        "rollout_entrypoint",
        "launched_at_unix_ms",
        "evidence_count",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("deployment_stack_launch_evidence_handoff_history.json"),
        "workflow reference must mention the deployment stack launch handoff history artifact"
    );
}

#[test]
fn deployment_stack_launch_handoff_history_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handoff = tmp
        .path()
        .join("deployment_stack_launch_evidence_handoff.json");
    fs::write(
        &handoff,
        r#"{
  "schema_version": "1",
  "bundle_path": "artifacts/deploy/deployment_stack_launch_evidence_bundle.json",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "deployable_unit": "afterburner-runtime-stack",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current",
  "launched_at_unix_ms": 1735689600000,
  "evidence_count": 2
}"#,
    )
    .expect("write handoff");
    let out = tmp
        .path()
        .join("deployment_stack_launch_evidence_handoff_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("launch")
        .arg("handoff")
        .arg("history")
        .arg("--handoff")
        .arg(&handoff)
        .arg("--event")
        .arg("handoff-updated")
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
    entry.insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "handoff-updated",
                    "handoff_path": "<handoff>",
                    "bundle_path": "artifacts/deploy/deployment_stack_launch_evidence_bundle.json",
                    "profile_name": "staging-a",
                    "deploy_hostname": "staging.example.net",
                    "deployable_unit": "afterburner-runtime-stack",
                    "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current",
                    "launched_at_unix_ms": 1735689600000_u64,
                    "evidence_count": 2,
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
                == Some("deployment_stack_launch_evidence_handoff_history_written")
        })
        .unwrap_or_else(|| {
            panic!("missing deployment stack launch handoff history event in stderr: {stderr}")
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
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_stack_launch_evidence_handoff_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}
