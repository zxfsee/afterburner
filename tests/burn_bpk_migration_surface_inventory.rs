use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn burn_bpk_migration_surface_inventory_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "burn_bpk_migration_surface_inventory.schema.json",
    ))
    .expect("read burn bpk surface schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse burn bpk surface schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/burn-bpk-migration-surface-inventory/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("required entries must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "current_burn_version",
        "observed_pre_release",
        "scan_scope",
        "touchpoints",
        "summary",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("burn-bpk-migration-surface-report:"),
        "justfile must expose the burn-bpk-migration-surface-report workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("burn_bpk_migration_surface_inventory.json"),
        "workflow reference must mention the burn `.bpk` surface inventory artifact"
    );
    assert!(
        workflows.contains("just burn-bpk-migration-surface-report"),
        "workflow reference must mention the burn `.bpk` surface inventory workflow"
    );

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("burn_bpk_migration_surface_inventory.json"),
        "reference index must mention the burn `.bpk` surface inventory artifact"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("burn_bpk_migration_surface_inventory.json"),
        "README must mention the burn `.bpk` surface inventory artifact"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("burn `.bpk` migration surface inventory"),
        "architecture must mention the burn `.bpk` migration surface inventory prerequisite"
    );

    let adr = repo_file("docs/adr/033-burn-dependency-refresh.md");
    assert!(
        adr.contains("burn_bpk_migration_surface_inventory.json"),
        "ADR-033 must mention the burn `.bpk` surface inventory report"
    );
}

#[test]
fn burn_bpk_migration_surface_inventory_writes_report_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("burn_bpk_migration_surface_inventory.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("inventory-burn-bpk-surface")
        .arg("--repo-root")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read inventory");
    let inventory: Value = serde_json::from_str(&text).expect("parse inventory");
    let expected_text = fs::read_to_string(fixture_path(
        "burn_bpk_migration_surface_inventory.example.json",
    ))
    .expect("read inventory example");
    let expected: Value = serde_json::from_str(&expected_text).expect("parse inventory example");
    assert_eq!(inventory, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("burn_bpk_migration_surface_inventory_written")
        })
        .unwrap_or_else(|| {
            panic!("missing burn bpk migration surface inventory event in stderr: {stderr}")
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
    fields.insert("inventory_path".to_string(), Value::from("<inventory>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "debug_cli",
            "event": "burn_bpk_migration_surface_inventory_written",
            "fields": {
                "inventory_path": "<inventory>",
                "inventory": expected
            }
        })
    );
}
