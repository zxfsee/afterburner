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
fn artifact_cleanup_execution_receipt_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path(
        "artifact_cleanup_execution_receipt.schema.json",
    ))
    .expect("read cleanup execution schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse cleanup execution schema fixture");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/artifact-cleanup-execution-receipt/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-029"),
        "architecture decisions index must link ADR-029"
    );
    assert!(
        architecture.contains("cleanup execution receipt"),
        "architecture must mention the cleanup execution receipt contract"
    );
    assert!(
        architecture.contains("artifact_cleanup_dry_run_receipt.json"),
        "architecture must anchor the execution receipt to the cleanup dry-run receipt"
    );

    let adr = repo_file("docs/adr/029-artifact-cleanup-execution-receipt.md");
    for needle in [
        "artifact_cleanup_execution_receipt.json",
        "artifact_cleanup_dry_run_receipt.json",
        "evidence_sources",
        "removed_paths",
        "skipped_paths",
        "executed_at_unix_ms",
        "policy_profile",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-029 must mention `{needle}` as part of the cleanup execution receipt contract"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("artifact_cleanup_execution_receipt.json"),
        "README must mention the cleanup execution receipt artifact"
    );
    assert!(
        readme.contains("just cleanup-execute"),
        "README must mention the cleanup execution workflow"
    );
}

#[test]
fn cleanup_execute_writes_receipt_and_removes_planned_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts_root = tmp.path().join("artifacts");
    fs::create_dir_all(artifacts_root.join("inference/0.1.0")).expect("create stale inference dir");
    fs::create_dir_all(artifacts_root.join("profiling")).expect("create profiling dir");
    fs::write(artifacts_root.join("profiling/summary.json"), "{}").expect("write profiling file");

    let dry_run_receipt = tmp.path().join("artifact_cleanup_dry_run_receipt.json");
    fs::write(
        &dry_run_receipt,
        format!(
            r#"{{
  "schema_version": "1",
  "inventory_path": "{}",
  "policy_path": "{}",
  "policy_profile": "retention-default",
  "planned_removals": ["inference/0.1.0", "profiling", "logs"],
  "retained_count": 5,
  "prune_candidate_count": 3,
  "generated_at_unix_ms": 1735689600000
}}"#,
            artifacts_root.join("deploy/inventory.json").display(),
            artifacts_root.join("deploy/policy.json").display()
        ),
    )
    .expect("write dry-run receipt fixture");

    let out = tmp.path().join("artifact_cleanup_execution_receipt.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("cleanup")
        .arg("execute")
        .arg("--dry-run-receipt")
        .arg(&dry_run_receipt)
        .arg("--artifacts-root")
        .arg(&artifacts_root)
        .arg("--executed-at-unix-ms")
        .arg("1735689605000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read cleanup execution receipt");
    let receipt: Value = serde_json::from_str(&text).expect("parse cleanup execution receipt");
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "dry_run_receipt_path": dry_run_receipt.display().to_string(),
            "policy_profile": "retention-default",
            "evidence_sources": [
                {
                    "artifact_path": dry_run_receipt.display().to_string(),
                    "artifact_role": "cleanup_dry_run_receipt",
                    "observed_at_unix_ms": 1735689600000_u64
                },
                {
                    "artifact_path": artifacts_root.join("deploy/inventory.json").display().to_string(),
                    "artifact_role": "cleanup_inventory",
                    "observed_at_unix_ms": 1735689600000_u64
                },
                {
                    "artifact_path": artifacts_root.join("deploy/policy.json").display().to_string(),
                    "artifact_role": "cleanup_policy",
                    "observed_at_unix_ms": 1735689600000_u64
                }
            ],
            "removed_paths": ["inference/0.1.0", "profiling"],
            "skipped_paths": ["logs"],
            "executed_at_unix_ms": 1735689605000_u64
        })
    );

    assert!(
        !artifacts_root.join("inference/0.1.0").exists(),
        "planned stale inference directory must be removed"
    );
    assert!(
        !artifacts_root.join("profiling").exists(),
        "planned profiling directory must be removed"
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("artifact_cleanup_execution_receipt_written")
        })
        .unwrap_or_else(|| panic!("missing execution receipt event in stderr: {stderr}"));
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
            "source": "ops_cli",
            "event": "artifact_cleanup_execution_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}
