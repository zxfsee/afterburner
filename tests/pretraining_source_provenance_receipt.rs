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
fn pretraining_source_provenance_receipt_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path(
        "pretraining_source_provenance_receipt.schema.json",
    ))
    .expect("read pretraining source provenance receipt schema fixture");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse pretraining source provenance receipt schema");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/pretraining-source-provenance-receipt/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-031"),
        "architecture decisions index must link ADR-031"
    );
    assert!(
        architecture.contains("source provenance receipt"),
        "architecture must mention the pretraining source provenance receipt contract"
    );
    assert!(
        architecture.contains("upstream_locator"),
        "architecture must anchor the provenance receipt to upstream_locator"
    );

    let adr = repo_file("docs/adr/031-pretraining-source-provenance-receipt.md");
    for needle in [
        "pretraining_source_approval_receipt.json",
        "registry_entry_path",
        "upstream_locator",
        "reviewed_metadata_sha256",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-031 must mention `{needle}` as part of source approval provenance"
        );
    }

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("source provenance receipt"),
        "workflow reference must mention the pretraining source provenance receipt contract"
    );
    assert!(
        readme.contains("just pretraining-source-provenance-receipt"),
        "workflow reference must mention the pretraining source provenance receipt workflow"
    );
}

#[test]
fn pretraining_source_provenance_receipt_writes_receipt_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let approval_receipt = tmp.path().join("pretraining_source_approval_receipt.json");
    fs::write(
        &approval_receipt,
        r#"{
  "schema_version": "1",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "approval_status": "approved",
  "approved_by": "ml-release",
  "approval_ticket": "CHG-4242",
  "approved_at_unix_ms": 1735689600000
}"#,
    )
    .expect("write pretraining source approval receipt");

    let out = tmp
        .path()
        .join("pretraining_source_provenance_receipt.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("source")
        .arg("provenance-receipt")
        .arg("--approval-receipt")
        .arg(&approval_receipt)
        .arg("--registry-entry-path")
        .arg("fixtures/source_registry/fineweb_edu.json")
        .arg("--upstream-locator")
        .arg("hf://datasets/HuggingFaceFW/fineweb-edu")
        .arg("--reviewed-metadata-sha256")
        .arg("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read pretraining source provenance receipt");
    let receipt: Value =
        serde_json::from_str(&text).expect("parse pretraining source provenance receipt");
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "approval_receipt_path": approval_receipt.display().to_string(),
            "registry_entry_path": "fixtures/source_registry/fineweb_edu.json",
            "upstream_locator": "hf://datasets/HuggingFaceFW/fineweb-edu",
            "reviewed_metadata_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("pretraining_source_provenance_receipt_written")
        })
        .unwrap_or_else(|| {
            panic!("missing pretraining source provenance receipt event in stderr: {stderr}")
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
            "event": "pretraining_source_provenance_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}
