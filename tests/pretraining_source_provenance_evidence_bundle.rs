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
fn pretraining_source_provenance_evidence_bundle_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "pretraining_source_provenance_evidence_bundle.schema.json",
    ))
    .expect("read source provenance evidence bundle schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse source provenance evidence bundle schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/pretraining-source-provenance-evidence-bundle/v1")
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
        "approval_receipt_path",
        "provenance_receipt_path",
        "source",
        "source_revision",
        "approval_status",
        "evidence_count",
        "evidence_sources",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("pretraining-source-provenance action +args:"),
        "justfile must expose the grouped pretraining-source-provenance workflow"
    );

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("pretraining_source_provenance_evidence_bundle.json"),
        "workflow reference must mention the pretraining source provenance evidence bundle artifact"
    );
    assert!(
        readme.contains("just pretraining-source-provenance <receipt|bundle>"),
        "workflow reference must mention the grouped pretraining source provenance workflow"
    );
}

#[test]
fn pretraining_source_provenance_evidence_bundle_writes_bundle_and_event() {
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
    .expect("write approval receipt");

    let provenance_receipt = tmp
        .path()
        .join("pretraining_source_provenance_receipt.json");
    fs::write(
        &provenance_receipt,
        format!(
            r#"{{
  "schema_version": "1",
  "approval_receipt_path": "{}",
  "registry_entry_path": "fixtures/source_registry/fineweb_edu.json",
  "upstream_locator": "hf://datasets/HuggingFaceFW/fineweb-edu",
  "reviewed_metadata_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}}"#,
            approval_receipt.display()
        ),
    )
    .expect("write provenance receipt");

    let out = tmp
        .path()
        .join("pretraining_source_provenance_evidence_bundle.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("source")
        .arg("provenance")
        .arg("bundle")
        .arg("--provenance-receipt")
        .arg(&provenance_receipt)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text =
        fs::read_to_string(&out).expect("read pretraining source provenance evidence bundle");
    let mut bundle: Value =
        serde_json::from_str(&text).expect("parse pretraining source provenance evidence bundle");
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert(
            "approval_receipt_path".to_string(),
            Value::from("<approval>"),
        );
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert(
            "provenance_receipt_path".to_string(),
            Value::from("<provenance>"),
        );
    assert_eq!(
        bundle,
        serde_json::json!({
            "schema_version": "1",
            "approval_receipt_path": "<approval>",
            "provenance_receipt_path": "<provenance>",
            "source": "fineweb-edu/slice",
            "source_revision": "2026-03-22-macbook-100m-tokens",
            "approval_status": "approved",
            "evidence_count": 1,
            "evidence_sources": [
                {
                    "registry_entry_path": "fixtures/source_registry/fineweb_edu.json",
                    "upstream_locator": "hf://datasets/HuggingFaceFW/fineweb-edu",
                    "reviewed_metadata_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
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
                == Some("pretraining_source_provenance_evidence_bundle_written")
        })
        .unwrap_or_else(|| {
            panic!("missing source provenance evidence bundle event in stderr: {stderr}")
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
    fields.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let bundle_obj = fields
        .get_mut("bundle")
        .and_then(Value::as_object_mut)
        .expect("event bundle must be an object");
    bundle_obj.insert(
        "approval_receipt_path".to_string(),
        Value::from("<approval>"),
    );
    bundle_obj.insert(
        "provenance_receipt_path".to_string(),
        Value::from("<provenance>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "data_cli",
            "event": "pretraining_source_provenance_evidence_bundle_written",
            "fields": {
                "bundle_path": "<bundle>",
                "bundle": bundle
            }
        })
    );
}
