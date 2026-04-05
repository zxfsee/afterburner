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
fn distributed_shard_lineage_transport_locator_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_transport_locator_reconciliation.schema.json",
    ))
    .expect("read lineage transport locator reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse lineage transport locator reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/distributed-shard-lineage-transport-locator-reconciliation/v1"
        )
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
        "handoff_path",
        "locator_path",
        "desired",
        "current",
        "reconciliation_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("distributed-shard-lineage-locator-transport-reconcile handoff locator:"),
        "justfile must expose the grouped distributed-shard-lineage-locator-transport reconcile workflow"
    );
}

#[test]
fn distributed_shard_lineage_transport_locator_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handoff = tmp
        .path()
        .join("distributed_shard_lineage_evidence_handoff.json");
    fs::write(
        &handoff,
        r#"{
  "schema_version": "1",
  "bundle_path": "artifacts/train/distributed_shard_lineage_evidence_bundle.json",
  "shard_id": "train-shard-0003",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "checkpoint_group": "ckpt-group-0007",
  "evidence_count": 1
}"#,
    )
    .expect("write handoff");
    let locator = tmp
        .path()
        .join("distributed_shard_lineage_transport_locator.json");
    fs::write(
        &locator,
        r#"{
  "schema_version": "1",
  "handoff_path": "artifacts/train/distributed_shard_lineage_evidence_handoff.json",
  "shard_id": "train-shard-0003",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "checkpoint_group": "ckpt-group-0007"
}"#,
    )
    .expect("write locator");
    let out = tmp
        .path()
        .join("distributed_shard_lineage_transport_locator_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("locator")
        .arg("transport")
        .arg("reconcile")
        .arg("--handoff")
        .arg(&handoff)
        .arg("--locator")
        .arg(&locator)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("locator_path".to_string(), Value::from("<locator>"));

    let expected_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_transport_locator_reconciliation.example.json",
    ))
    .expect("read lineage transport locator reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse lineage transport locator reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("distributed_shard_lineage_transport_locator_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!("missing lineage transport locator reconciliation event in stderr: {stderr}")
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
    recon_obj.insert("handoff_path".to_string(), Value::from("<handoff>"));
    recon_obj.insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_transport_locator_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
