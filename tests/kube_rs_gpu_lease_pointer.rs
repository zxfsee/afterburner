use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn kube_rs_gpu_lease_pointer_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("kube_rs_gpu_lease_pointer.schema.json"))
        .expect("read kube-rs lease pointer schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse kube-rs lease pointer");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/kube-rs-gpu-lease-pointer/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required entries must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "reconciliation_path",
        "lease_id",
        "job_id",
        "node_id",
        "gpu_ids",
        "checkpoint_mode",
        "namespace",
        "resource_name",
        "desired_state",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("kube_rs_gpu_lease_pointer.json"),
        "workflow reference must mention the kube-rs lease pointer artifact"
    );
}

#[test]
fn kube_rs_gpu_lease_pointer_writes_pointer_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("kube_rs_gpu_lease_pointer.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("point-kube-rs-lease")
        .arg("--reconciliation")
        .arg(fixture_path(
            "kube_rs_gpu_lease_reconciliation.example.json",
        ))
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read kube-rs lease pointer");
    let mut pointer: Value = serde_json::from_str(&text).expect("parse kube-rs lease pointer");
    pointer
        .as_object_mut()
        .expect("pointer must be object")
        .insert(
            "reconciliation_path".to_string(),
            Value::from("<reconciliation>"),
        );
    let expected_text = fs::read_to_string(fixture_path("kube_rs_gpu_lease_pointer.example.json"))
        .expect("read kube-rs lease pointer example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse kube-rs lease pointer example");
    expected
        .as_object_mut()
        .expect("expected pointer must be object")
        .insert(
            "reconciliation_path".to_string(),
            Value::from("<reconciliation>"),
        );
    assert_eq!(pointer, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str) == Some("kube_rs_gpu_lease_pointer_written")
        })
        .unwrap_or_else(|| panic!("missing kube-rs lease pointer event in stderr: {stderr}"));
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("pointer_path".to_string(), Value::from("<pointer>"));
    let pointer_obj = fields
        .get_mut("pointer")
        .and_then(Value::as_object_mut)
        .expect("event pointer must be an object");
    pointer_obj.insert(
        "reconciliation_path".to_string(),
        Value::from("<reconciliation>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "kube_rs_gpu_lease_pointer_written",
            "fields": {
                "pointer_path": "<pointer>",
                "pointer": expected
            }
        })
    );
}
