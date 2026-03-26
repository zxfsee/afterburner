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
fn distributed_shard_lineage_handoff_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_evidence_handoff.schema.json",
    ))
    .expect("read lineage handoff schema fixture");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse lineage handoff schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-shard-lineage-evidence-handoff/v1")
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
        "bundle_path",
        "shard_id",
        "source",
        "source_revision",
        "checkpoint_group",
        "evidence_count",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("distributed-shard-lineage-handoff bundle:"),
        "justfile must expose the distributed-shard-lineage-handoff workflow"
    );

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("distributed_shard_lineage_evidence_handoff.json"),
        "workflow reference must mention the distributed shard lineage evidence handoff artifact"
    );
    assert!(
        readme.contains("just distributed-shard-lineage-handoff"),
        "workflow reference must mention the distributed shard lineage evidence handoff workflow"
    );
}

#[test]
fn distributed_shard_lineage_handoff_writes_handoff_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp
        .path()
        .join("distributed_shard_lineage_evidence_bundle.json");
    fs::write(
        &bundle,
        r#"{
  "schema_version": "1",
  "receipt_path": "artifacts/train/distributed_shard_lineage_receipt.json",
  "shard_id": "train-shard-0003",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "checkpoint_group": "ckpt-group-0007",
  "evidence_count": 1,
  "evidence_sources": [
    {"metadata_path":"fixtures/distributed_shard_metadata.example.json","checkpoint_root":"artifacts/train/checkpoints/group-0007","observed_at_unix_ms":1735689600000}
  ]
}"#,
    )
    .expect("write lineage evidence bundle");

    let out = tmp
        .path()
        .join("distributed_shard_lineage_evidence_handoff.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("handoff")
        .arg("--bundle")
        .arg(&bundle)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read lineage handoff");
    let mut handoff: Value = serde_json::from_str(&text).expect("parse lineage handoff");
    handoff
        .as_object_mut()
        .expect("handoff must be object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        handoff,
        serde_json::json!({
            "schema_version": "1",
            "bundle_path": "<bundle>",
            "shard_id": "train-shard-0003",
            "source": "fineweb-edu/slice",
            "source_revision": "2026-03-22-macbook-100m-tokens",
            "checkpoint_group": "ckpt-group-0007",
            "evidence_count": 1
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("distributed_shard_lineage_evidence_handoff_written")
        })
        .unwrap_or_else(|| panic!("missing lineage handoff event in stderr: {stderr}"));
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("handoff_path".to_string(), Value::from("<handoff>"));
    let handoff_obj = fields
        .get_mut("handoff")
        .and_then(Value::as_object_mut)
        .expect("event handoff must be an object");
    handoff_obj.insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_evidence_handoff_written",
            "fields": {
                "handoff_path": "<handoff>",
                "handoff": handoff
            }
        })
    );
}
