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
fn artifact_cleanup_evidence_bundle_schema_and_workflow_are_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("artifact_cleanup_evidence_bundle.schema.json"))
            .expect("read cleanup evidence bundle schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse cleanup evidence bundle schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/artifact-cleanup-evidence-bundle/v1")
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
        "dry_run_receipt_path",
        "execution_receipt_path",
        "policy_profile",
        "removed_count",
        "skipped_count",
        "evidence_count",
        "evidence_sources",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("cleanup-evidence-bundle execution_receipt:"),
        "justfile must expose the cleanup-evidence-bundle workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("artifact_cleanup_evidence_bundle.json"),
        "workflow reference must mention the cleanup evidence bundle artifact"
    );
    assert!(
        workflows.contains("just cleanup-evidence-bundle"),
        "workflow reference must mention the cleanup evidence bundle workflow"
    );
}

#[test]
fn cleanup_evidence_bundle_writes_bundle_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let execution_receipt = tmp.path().join("artifact_cleanup_execution_receipt.json");
    fs::write(
        &execution_receipt,
        r#"{
  "schema_version": "1",
  "dry_run_receipt_path": "/tmp/artifacts/deploy/artifact_cleanup_dry_run_receipt.json",
  "policy_profile": "retention-default",
  "evidence_sources": [
    {"artifact_path": "/tmp/artifacts/deploy/artifact_cleanup_dry_run_receipt.json", "artifact_role": "cleanup_dry_run_receipt", "observed_at_unix_ms": 1735689600000},
    {"artifact_path": "/tmp/artifacts/deploy/artifact_cleanup_inventory.json", "artifact_role": "cleanup_inventory", "observed_at_unix_ms": 1735689600000},
    {"artifact_path": "/tmp/artifacts/deploy/artifact_cleanup_policy.json", "artifact_role": "cleanup_policy", "observed_at_unix_ms": 1735689600000}
  ],
  "removed_paths": ["inference/0.1.0", "profiling"],
  "skipped_paths": ["logs"],
  "executed_at_unix_ms": 1735689605000
}"#,
    )
    .expect("write cleanup execution receipt");

    let out = tmp.path().join("artifact_cleanup_evidence_bundle.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("cleanup")
        .arg("evidence-bundle")
        .arg("--execution-receipt")
        .arg(&execution_receipt)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read cleanup evidence bundle");
    let mut bundle: Value = serde_json::from_str(&text).expect("parse cleanup evidence bundle");
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert(
            "execution_receipt_path".to_string(),
            Value::from("<execution-receipt>"),
        );
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert(
            "dry_run_receipt_path".to_string(),
            Value::from("<dry-run-receipt>"),
        );
    assert_eq!(
        bundle,
        serde_json::json!({
            "schema_version": "1",
            "dry_run_receipt_path": "<dry-run-receipt>",
            "execution_receipt_path": "<execution-receipt>",
            "policy_profile": "retention-default",
            "removed_count": 2,
            "skipped_count": 1,
            "evidence_count": 3,
            "evidence_sources": [
                {
                    "artifact_path": "/tmp/artifacts/deploy/artifact_cleanup_dry_run_receipt.json",
                    "artifact_role": "cleanup_dry_run_receipt",
                    "observed_at_unix_ms": 1735689600000i64
                },
                {
                    "artifact_path": "/tmp/artifacts/deploy/artifact_cleanup_inventory.json",
                    "artifact_role": "cleanup_inventory",
                    "observed_at_unix_ms": 1735689600000i64
                },
                {
                    "artifact_path": "/tmp/artifacts/deploy/artifact_cleanup_policy.json",
                    "artifact_role": "cleanup_policy",
                    "observed_at_unix_ms": 1735689600000i64
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
                == Some("artifact_cleanup_evidence_bundle_written")
        })
        .unwrap_or_else(|| panic!("missing cleanup evidence bundle event in stderr: {stderr}"));
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let bundle_obj = fields
        .get_mut("bundle")
        .and_then(Value::as_object_mut)
        .expect("event bundle must be an object");
    bundle_obj.insert(
        "execution_receipt_path".to_string(),
        Value::from("<execution-receipt>"),
    );
    bundle_obj.insert(
        "dry_run_receipt_path".to_string(),
        Value::from("<dry-run-receipt>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "ops_cli",
            "event": "artifact_cleanup_evidence_bundle_written",
            "fields": {
                "bundle_path": "<bundle>",
                "bundle": bundle
            }
        })
    );
}
