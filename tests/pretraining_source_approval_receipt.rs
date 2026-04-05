use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn pretraining_source_approval_receipt_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path(
        "pretraining_source_approval_receipt.schema.json",
    ))
    .expect("read pretraining source approval receipt schema fixture");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse pretraining source approval receipt schema");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/pretraining-source-approval-receipt/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-026"),
        "architecture decisions index must link ADR-026"
    );
    let adr = repo_file("docs/adr/026-pretraining-source-approval-receipt.md");
    for needle in [
        "pretraining_source_approval_receipt.json",
        "source",
        "source_revision",
        "approval_status",
        "approved_by",
        "approval_ticket",
        "approved_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-026 must mention `{needle}` as part of the source approval receipt contract"
        );
    }

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("pretraining_source_approval_receipt.json"),
        "workflow reference must mention the pretraining source approval receipt artifact"
    );
    assert!(
        readme.contains("just pretraining-source-approval"),
        "workflow reference must mention the grouped pretraining source approval workflow"
    );
    let reference = repo_file("docs/reference.md");
    for needle in ["source approval receipt", "approval_status"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the source approval anchor `{needle}`"
        );
    }
}

#[test]
fn pretraining_source_approval_receipt_writes_receipt_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("pretraining_source_approval_receipt.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("source")
        .arg("approval")
        .arg("receipt")
        .arg("--source")
        .arg("fineweb-edu/slice")
        .arg("--source-revision")
        .arg("2026-03-22-macbook-100m-tokens")
        .arg("--approval-status")
        .arg("approved")
        .arg("--approved-by")
        .arg("ml-release")
        .arg("--approval-ticket")
        .arg("CHG-4242")
        .arg("--approved-at-unix-ms")
        .arg("1735689600000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read pretraining source approval receipt");
    let receipt: Value =
        serde_json::from_str(&text).expect("parse pretraining source approval receipt");
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "source": "fineweb-edu/slice",
            "source_revision": "2026-03-22-macbook-100m-tokens",
            "approval_status": "approved",
            "approved_by": "ml-release",
            "approval_ticket": "CHG-4242",
            "approved_at_unix_ms": 1735689600000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("pretraining_source_approval_receipt_written")
        })
        .unwrap_or_else(|| {
            panic!("missing pretraining source approval receipt event in stderr: {stderr}")
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
            "source": "data_cli",
            "event": "pretraining_source_approval_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}
