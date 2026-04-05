use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn artifact_cleanup_policy_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("artifact_cleanup_policy.schema.json"))
        .expect("read cleanup policy schema fixture");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse cleanup policy schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/artifact-cleanup-policy/v1")
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
        "profile_name",
        "protected_categories",
        "prune_candidate_categories",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("cleanup-policy profile:"),
        "justfile must expose the cleanup-policy workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("artifact_cleanup_policy.json"),
        "workflow reference must mention the cleanup policy artifact"
    );
    assert!(
        workflows.contains("just cleanup-policy"),
        "workflow reference must mention the cleanup policy workflow"
    );
}

#[test]
fn cleanup_policy_writes_named_policy_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("artifact_cleanup_policy.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("cleanup")
        .arg("policy")
        .arg("--profile")
        .arg("retention-default")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read cleanup policy");
    let policy: Value = serde_json::from_str(&text).expect("parse cleanup policy");
    assert_eq!(
        policy,
        serde_json::json!({
            "schema_version": "1",
            "profile_name": "retention-default",
            "protected_categories": [
                "runtime_pointer",
                "active_inference_directory",
                "rollout_evidence",
                "train_contracts",
                "eval_state"
            ],
            "prune_candidate_categories": [
                "inactive_inference_directory",
                "profiling_artifacts"
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "artifact_cleanup_policy_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("policy_path".to_string(), Value::from("<policy>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "ops_cli",
            "event": "artifact_cleanup_policy_written",
            "fields": {
                "policy_path": "<policy>",
                "policy": policy
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
