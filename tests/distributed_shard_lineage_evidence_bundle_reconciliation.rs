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
fn distributed_shard_lineage_evidence_bundle_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_evidence_bundle_reconciliation.schema.json",
    ))
    .expect("read lineage evidence bundle reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse lineage evidence bundle reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/distributed-shard-lineage-evidence-bundle-reconciliation/v1"
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
        "receipt_path",
        "bundle_path",
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
        justfile.contains("distributed-shard-lineage-bundle-reconcile receipt bundle:"),
        "justfile must expose the distributed-shard-lineage-bundle-reconcile workflow"
    );
}

#[test]
fn distributed_shard_lineage_evidence_bundle_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let receipt = tmp.path().join("distributed_shard_lineage_receipt.json");
    fs::write(
        &receipt,
        r#"{
  "schema_version": "1",
  "shard_id": "train-shard-0003",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "checkpoint_group": "ckpt-group-0007",
  "checked_at_unix_ms": 1735689600000,
  "evidence_sources": [
    {
      "metadata_path": "fixtures/distributed_shard_metadata.example.json",
      "checkpoint_root": "artifacts/train/checkpoints/group-0007",
      "observed_at_unix_ms": 1735689600000
    }
  ]
}"#,
    )
    .expect("write receipt");
    let bundle = tmp
        .path()
        .join("distributed_shard_lineage_evidence_bundle.json");
    fs::write(
        &bundle,
        format!(
            r#"{{
  "schema_version": "1",
  "receipt_path": "{}",
  "shard_id": "train-shard-0003",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "checkpoint_group": "ckpt-group-0007",
  "evidence_count": 1,
  "evidence_sources": [
    {{
      "metadata_path": "fixtures/distributed_shard_metadata.example.json",
      "checkpoint_root": "artifacts/train/checkpoints/group-0007",
      "observed_at_unix_ms": 1735689600000
    }}
  ]
}}"#,
            receipt.display()
        ),
    )
    .expect("write bundle");
    let out = tmp
        .path()
        .join("distributed_shard_lineage_evidence_bundle_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("bundle")
        .arg("reconcile")
        .arg("--receipt")
        .arg(&receipt)
        .arg("--bundle")
        .arg(&bundle)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("receipt_path".to_string(), Value::from("<receipt>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    let recon_obj = reconciliation.as_object_mut().expect("object");
    let desired = recon_obj
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("desired object");
    desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let current = recon_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("current object");
    current.insert("receipt_path".to_string(), Value::from("<receipt>"));

    let expected_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_evidence_bundle_reconciliation.example.json",
    ))
    .expect("read lineage bundle reconciliation example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse lineage bundle reconciliation example");
    let expected_obj = expected.as_object_mut().expect("expected object");
    expected_obj.insert("receipt_path".to_string(), Value::from("<receipt>"));
    expected_obj.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let expected_desired = expected_obj
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("expected desired");
    expected_desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let expected_current = expected_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("expected current");
    expected_current.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("distributed_shard_lineage_evidence_bundle_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!("missing lineage bundle reconciliation event in stderr: {stderr}")
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
    let event_recon = fields
        .get_mut("reconciliation")
        .and_then(Value::as_object_mut)
        .expect("event reconciliation object");
    event_recon.insert("receipt_path".to_string(), Value::from("<receipt>"));
    event_recon.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let event_desired = event_recon
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("event desired");
    event_desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let event_current = event_recon
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("event current");
    event_current.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_evidence_bundle_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
