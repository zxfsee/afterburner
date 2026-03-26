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
fn artifact_cleanup_dry_run_receipt_contract_is_documented() {
    let schema_text =
        fs::read_to_string(fixture_path("artifact_cleanup_dry_run_receipt.schema.json"))
            .expect("read cleanup dry-run schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse cleanup dry-run schema fixture");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/artifact-cleanup-dry-run-receipt/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-024"),
        "architecture decisions index must link ADR-024"
    );
    assert!(
        architecture.contains("cleanup dry-run receipt"),
        "architecture must mention the cleanup dry-run receipt contract"
    );
    assert!(
        architecture.contains("artifact_cleanup_inventory.json"),
        "architecture must anchor the dry-run receipt to the cleanup inventory artifact"
    );

    let adr = repo_file("docs/adr/024-artifact-cleanup-dry-run-receipt.md");
    for needle in [
        "artifact_cleanup_dry_run_receipt.json",
        "artifact_cleanup_inventory.json",
        "planned_removals",
        "retained_count",
        "prune_candidate_count",
        "generated_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-024 must mention `{needle}` as part of the dry-run receipt contract"
        );
    }

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("artifact_cleanup_dry_run_receipt.json"),
        "workflow reference must mention the cleanup dry-run receipt artifact"
    );
    assert!(
        readme.contains("just cleanup-dry-run"),
        "workflow reference must mention the cleanup dry-run workflow"
    );
}

#[test]
fn cleanup_dry_run_writes_receipt_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let inventory = tmp.path().join("artifact_cleanup_inventory.json");
    let policy = tmp.path().join("artifact_cleanup_policy.json");
    let out = tmp.path().join("artifact_cleanup_dry_run_receipt.json");

    fs::write(
        &inventory,
        r#"{
  "schema_version": "1",
  "artifacts_root": "/tmp/artifacts",
  "active_inference_version": "0.2.0",
  "retained": [
    {"path":"inference/current","category":"runtime_pointer","reason":"active inference pointer must be retained"},
    {"path":"inference/0.2.0","category":"active_inference_directory","reason":"referenced by current inference pointer"},
    {"path":"deploy","category":"rollout_evidence","reason":"active rollout evidence stays retained"},
    {"path":"train","category":"train_contracts","reason":"train-side decision artifacts stay retained"},
    {"path":"eval","category":"eval_state","reason":"current eval and drift state stays retained"}
  ],
  "prune_candidates": [
    {"path":"inference/0.1.0","category":"inactive_inference_directory","reason":"not referenced by current inference pointer"},
    {"path":"profiling","category":"profiling_artifacts","reason":"profiling artifacts are prune candidates once no active runtime or rollout state references them"}
  ]
}"#,
    )
    .expect("write inventory fixture");

    fs::write(
        &policy,
        r#"{
  "schema_version": "1",
  "profile_name": "retention-default",
  "protected_categories": ["runtime_pointer","active_inference_directory","rollout_evidence","train_contracts","eval_state"],
  "prune_candidate_categories": ["inactive_inference_directory","profiling_artifacts"]
}"#,
    )
    .expect("write policy fixture");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("cleanup")
        .arg("dry-run")
        .arg("--inventory")
        .arg(&inventory)
        .arg("--policy")
        .arg(&policy)
        .arg("--generated-at-unix-ms")
        .arg("1735689600000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read cleanup dry-run receipt");
    let receipt: Value = serde_json::from_str(&text).expect("parse cleanup dry-run receipt");
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "inventory_path": inventory.display().to_string(),
            "policy_path": policy.display().to_string(),
            "policy_profile": "retention-default",
            "planned_removals": ["inference/0.1.0", "profiling"],
            "retained_count": 5,
            "prune_candidate_count": 2,
            "generated_at_unix_ms": 1735689600000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("artifact_cleanup_dry_run_receipt_written")
        })
        .unwrap_or_else(|| panic!("missing dry-run receipt event in stderr: {stderr}"));
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
            "event": "artifact_cleanup_dry_run_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}
