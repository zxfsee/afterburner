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
fn artifact_cleanup_inventory_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("artifact_cleanup_inventory.schema.json"))
        .expect("read cleanup inventory schema fixture");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse cleanup inventory schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/artifact-cleanup-inventory/v1")
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
        "artifacts_root",
        "active_inference_version",
        "retained",
        "prune_candidates",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("cleanup-inventory:"),
        "justfile must expose the cleanup-inventory workflow"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("artifact_cleanup_inventory.json"),
        "README must mention the cleanup inventory artifact"
    );
    assert!(
        readme.contains("just cleanup-inventory"),
        "README must mention the cleanup inventory workflow"
    );
}

#[test]
fn cleanup_inventory_writes_inventory_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts_root = tmp.path().join("artifacts");
    fs::create_dir_all(artifacts_root.join("inference/0.2.0")).expect("create active version dir");
    fs::create_dir_all(artifacts_root.join("inference/0.1.0"))
        .expect("create inactive version dir");
    fs::create_dir_all(artifacts_root.join("deploy")).expect("create deploy dir");
    fs::create_dir_all(artifacts_root.join("train")).expect("create train dir");
    fs::create_dir_all(artifacts_root.join("eval")).expect("create eval dir");
    fs::create_dir_all(artifacts_root.join("profiling")).expect("create profiling dir");
    fs::write(artifacts_root.join("inference/current"), "0.2.0\n").expect("write current version");
    fs::write(artifacts_root.join("deploy/check.txt"), "ok").expect("write deploy artifact");
    fs::write(artifacts_root.join("train/observability.jsonl"), "{}\n")
        .expect("write train artifact");
    fs::write(artifacts_root.join("eval/mnist_eval_summary.json"), "{}")
        .expect("write eval artifact");
    fs::write(
        artifacts_root.join("profiling/infer_hotspot_summary.json"),
        "{}",
    )
    .expect("write profiling artifact");

    let out = tmp.path().join("artifact_cleanup_inventory.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("cleanup")
        .arg("inventory")
        .arg("--artifacts-root")
        .arg(&artifacts_root)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read cleanup inventory");
    let mut inventory: Value = serde_json::from_str(&text).expect("parse cleanup inventory");
    inventory
        .as_object_mut()
        .expect("inventory must be object")
        .insert(
            "artifacts_root".to_string(),
            Value::from("<artifacts-root>"),
        );
    assert_eq!(
        inventory,
        serde_json::json!({
            "schema_version": "1",
            "artifacts_root": "<artifacts-root>",
            "active_inference_version": "0.2.0",
            "retained": [
                {
                    "path": "deploy",
                    "category": "rollout_evidence",
                    "reason": "active rollout evidence stays retained"
                },
                {
                    "path": "eval",
                    "category": "eval_state",
                    "reason": "current eval and drift state stays retained"
                },
                {
                    "path": "inference/0.2.0",
                    "category": "active_inference_directory",
                    "reason": "referenced by current inference pointer"
                },
                {
                    "path": "inference/current",
                    "category": "runtime_pointer",
                    "reason": "active inference pointer must be retained"
                },
                {
                    "path": "train",
                    "category": "train_contracts",
                    "reason": "train-side decision artifacts stay retained"
                }
            ],
            "prune_candidates": [
                {
                    "path": "inference/0.1.0",
                    "category": "inactive_inference_directory",
                    "reason": "not referenced by current inference pointer"
                },
                {
                    "path": "profiling",
                    "category": "profiling_artifacts",
                    "reason": "profiling artifacts are prune candidates once no active runtime or rollout state references them"
                }
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "artifact_cleanup_inventory_written");
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
    let inventory_obj = fields
        .get_mut("inventory")
        .and_then(Value::as_object_mut)
        .expect("event inventory must be an object");
    inventory_obj.insert(
        "artifacts_root".to_string(),
        Value::from("<artifacts-root>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "ops_cli",
            "event": "artifact_cleanup_inventory_written",
            "fields": {
                "inventory_path": "<inventory>",
                "inventory": inventory
            }
        })
    );
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}
