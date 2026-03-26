use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn distributed_shard_lineage_receipt_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_receipt.schema.json",
    ))
    .expect("read distributed shard lineage receipt schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse distributed shard lineage receipt schema");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-shard-lineage-receipt/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-028"),
        "architecture decisions index must link ADR-028"
    );
    assert!(
        architecture.contains("distributed shard lineage receipt"),
        "architecture must mention the distributed shard lineage receipt contract"
    );
    assert!(
        architecture.contains("checkpoint_group"),
        "architecture must anchor the lineage receipt to checkpoint_group"
    );

    let adr = repo_file("docs/adr/028-distributed-shard-lineage-receipt.md");
    for needle in [
        "distributed_shard_lineage_receipt.json",
        "shard_id",
        "source",
        "source_revision",
        "checkpoint_group",
        "checked_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-028 must mention `{needle}` as part of the lineage receipt contract"
        );
    }

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("distributed_shard_lineage_receipt.json"),
        "workflow reference must mention the distributed shard lineage receipt artifact"
    );
    assert!(
        readme.contains("just distributed-shard-lineage-receipt"),
        "workflow reference must mention the distributed shard lineage receipt workflow"
    );
}

#[test]
fn distributed_shard_lineage_receipt_writes_receipt_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let metadata = tmp.path().join("distributed_shard_metadata.json");
    fs::write(
        &metadata,
        r#"{
  "schema_version": "1",
  "shard_id": "train-shard-0003",
  "shard_index": 3,
  "shard_count": 8,
  "owner_worker_id": "worker-07",
  "owner_worker_rank": 7,
  "checksum_algorithm": "sha256",
  "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}"#,
    )
    .expect("write distributed shard metadata");

    let out = tmp.path().join("distributed_shard_lineage_receipt.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("receipt")
        .arg("--metadata")
        .arg(&metadata)
        .arg("--shard-id")
        .arg("train-shard-0003")
        .arg("--source")
        .arg("fineweb-edu/slice")
        .arg("--source-revision")
        .arg("2026-03-22-macbook-100m-tokens")
        .arg("--checkpoint-group")
        .arg("ckpt-group-0007")
        .arg("--checkpoint-root")
        .arg("artifacts/train/checkpoints/group-0007")
        .arg("--checked-at-unix-ms")
        .arg("1735689600000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read distributed shard lineage receipt");
    let receipt: Value =
        serde_json::from_str(&text).expect("parse distributed shard lineage receipt");
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "shard_id": "train-shard-0003",
            "source": "fineweb-edu/slice",
            "source_revision": "2026-03-22-macbook-100m-tokens",
            "checkpoint_group": "ckpt-group-0007",
            "checked_at_unix_ms": 1735689600000_u64,
            "evidence_sources": [
                {
                    "metadata_path": metadata.display().to_string(),
                    "checkpoint_root": "artifacts/train/checkpoints/group-0007",
                    "observed_at_unix_ms": 1735689600000_u64
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
                == Some("distributed_shard_lineage_receipt_written")
        })
        .unwrap_or_else(|| {
            panic!("missing distributed shard lineage receipt event in stderr: {stderr}")
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
    fields.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}
