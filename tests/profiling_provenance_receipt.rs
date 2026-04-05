use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn profiling_provenance_receipt_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path("profiling_provenance_receipt.schema.json"))
        .expect("read profiling provenance receipt schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse profiling provenance receipt schema");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/profiling-provenance-receipt/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-030"),
        "architecture decisions index must link ADR-030"
    );
    let adr = repo_file("docs/adr/030-profiling-provenance-receipt.md");
    for needle in [
        "profiling_provenance_receipt.json",
        "profiling_environment_snapshot.json",
        "profile_kind",
        "profiler_version",
        "captured_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-030 must mention `{needle}` as part of the profiling provenance receipt contract"
        );
    }

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("profiling_provenance_receipt.json"),
        "workflow reference must mention the profiling provenance receipt artifact"
    );
    assert!(
        reference.contains("profiling_environment_snapshot.json"),
        "reference index must mention the profiling provenance receipt anchor"
    );
    assert!(
        workflows.contains("just profile-provenance-receipt"),
        "workflow reference must mention the profiling provenance receipt workflow"
    );
}

#[test]
fn profiling_provenance_receipt_writes_receipt_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let snapshot = tmp.path().join("profiling_environment_snapshot.json");
    fs::write(
        &snapshot,
        r#"{
  "schema_version": "1",
  "profile_kind": "infer",
  "profiler": "cargo-flamegraph",
  "profiler_path": "/tmp/fake-profiler",
  "profiler_version": "fake-profiler 1.2.3",
  "host_os": "darwin",
  "host_arch": "aarch64"
}"#,
    )
    .expect("write profiling snapshot");

    let refresh_receipt = tmp
        .path()
        .join("profiling_environment_snapshot_refresh.json");
    fs::write(
        &refresh_receipt,
        r#"{
  "schema_version": "1",
  "previous_snapshot_path": "/tmp/prev.json",
  "refreshed_snapshot_path": "/tmp/current.json",
  "profile_kind": "infer",
  "profiler": "cargo-flamegraph",
  "previous_profiler_path": "/tmp/old-profiler",
  "refreshed_profiler_path": "/tmp/fake-profiler",
  "previous_profiler_version": "old",
  "refreshed_profiler_version": "fake-profiler 1.2.3",
  "captured_at_unix_ms": 1735689600000
}"#,
    )
    .expect("write refresh receipt");

    let out = tmp.path().join("profiling_provenance_receipt.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("profile")
        .arg("provenance-receipt")
        .arg("--snapshot")
        .arg(&snapshot)
        .arg("--captured-at-unix-ms")
        .arg("1735689600000")
        .arg("--refresh-receipt")
        .arg(&refresh_receipt)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read profiling provenance receipt");
    let mut receipt: Value =
        serde_json::from_str(&text).expect("parse profiling provenance receipt");
    receipt
        .as_object_mut()
        .expect("receipt must be object")
        .insert("snapshot_path".to_string(), Value::from("<snapshot>"));
    receipt
        .as_object_mut()
        .expect("receipt must be object")
        .insert("refresh_receipt_path".to_string(), Value::from("<refresh>"));
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "snapshot_path": "<snapshot>",
            "refresh_receipt_path": "<refresh>",
            "profile_kind": "infer",
            "profiler_version": "fake-profiler 1.2.3",
            "captured_at_unix_ms": 1735689600000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("profiling_provenance_receipt_written")
        })
        .unwrap_or_else(|| {
            panic!("missing profiling provenance receipt event in stderr: {stderr}")
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
    let receipt_obj = fields
        .get_mut("receipt")
        .and_then(Value::as_object_mut)
        .expect("event receipt must be an object");
    receipt_obj.insert("snapshot_path".to_string(), Value::from("<snapshot>"));
    receipt_obj.insert("refresh_receipt_path".to_string(), Value::from("<refresh>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "profiling_cli",
            "event": "profiling_provenance_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}
