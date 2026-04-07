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
fn kube_rs_gpu_lease_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("kube_rs_gpu_lease_reconciliation.schema.json"))
            .expect("read kube-rs lease reconciliation schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse kube-rs lease reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/kube-rs-gpu-lease-reconciliation/v1")
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
        "lease_path",
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

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("kube-rs-lease-reconcile lease namespace resource_name:"),
        "justfile must expose the kube-rs-lease-reconcile workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("kube_rs_gpu_lease_reconciliation.json"),
        "workflow reference must mention the kube-rs lease reconciliation artifact"
    );
    assert!(
        workflows.contains("just kube-rs-lease-reconcile"),
        "workflow reference must mention the kube-rs lease reconciliation workflow"
    );
}

#[test]
fn kube_rs_gpu_lease_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("kube_rs_gpu_lease_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("lease")
        .arg("reconcile")
        .arg("--lease")
        .arg(fixture_path("gpu_scheduler_lease.example.json"))
        .arg("--namespace")
        .arg("afterburner-system")
        .arg("--resource-name")
        .arg("lease-job-0001")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read kube-rs lease reconciliation");
    let mut reconciliation: Value =
        serde_json::from_str(&text).expect("parse kube-rs lease reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("lease_path".to_string(), Value::from("<lease>"));
    let expected_text = fs::read_to_string(fixture_path(
        "kube_rs_gpu_lease_reconciliation.example.json",
    ))
    .expect("read kube-rs lease reconciliation example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse kube-rs lease reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("lease_path".to_string(), Value::from("<lease>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("kube_rs_gpu_lease_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!("missing kube-rs lease reconciliation event in stderr: {stderr}")
        });
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert(
        "reconciliation_path".to_string(),
        Value::from("<reconciliation>"),
    );
    let recon_obj = fields
        .get_mut("reconciliation")
        .and_then(Value::as_object_mut)
        .expect("event reconciliation must be an object");
    recon_obj.insert("lease_path".to_string(), Value::from("<lease>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "kube_rs_gpu_lease_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
