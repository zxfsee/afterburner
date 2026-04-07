use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn distributed_runtime_checkpoint_state_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_runtime_checkpoint_state.schema.json",
    ))
    .expect("read checkpoint state schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-runtime-checkpoint-state/v1")
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("required values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "artifact_version",
        "backend",
        "checkpoint_group",
        "checkpoint_root",
        "resume_epoch",
        "world_size",
        "device_group",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("distributed_runtime_checkpoint_state.json"),
        "architecture must mention the runtime checkpoint-state artifact"
    );

    let reference = repo_file("docs/reference.md");
    for needle in [
        "distributed_runtime_checkpoint_state.json",
        "checkpoint_group",
        "checkpoint_root",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must mention `{needle}` for runtime checkpoint state"
        );
    }
}

#[test]
fn train_runtime_checkpoint_group_anchor_and_resume_are_executable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts_dir = tmp.path().join("artifacts");

    let mut initial = cargo_bin_cmd!("afterburner");
    initial
        .env("BACKEND", "cpu")
        .env("ARTIFACTS_DIR", &artifacts_dir)
        .env("ARTIFACT_VERSION", "0.3.0-runtime")
        .arg("train")
        .arg("--batch-size")
        .arg("8")
        .arg("--num-epochs")
        .arg("1")
        .arg("--max-train-items")
        .arg("16")
        .arg("--max-valid-items")
        .arg("8")
        .arg("--checkpoint-group")
        .arg("group-a");
    initial.assert().success();

    let mut resumed = cargo_bin_cmd!("afterburner");
    let resumed_assert = resumed
        .env("BACKEND", "cpu")
        .env("ARTIFACTS_DIR", &artifacts_dir)
        .env("ARTIFACT_VERSION", "0.3.0-runtime")
        .arg("train")
        .arg("--batch-size")
        .arg("8")
        .arg("--num-epochs")
        .arg("2")
        .arg("--max-train-items")
        .arg("16")
        .arg("--max-valid-items")
        .arg("8")
        .arg("--checkpoint-group")
        .arg("group-a")
        .arg("--resume-epoch")
        .arg("1")
        .assert()
        .success();

    let checkpoint_state_path = artifacts_dir
        .join("train")
        .join("distributed_runtime_checkpoint_state.json");
    assert!(
        checkpoint_state_path.is_file(),
        "runtime checkpoint state artifact must be written"
    );

    let checkpoint_root = artifacts_dir
        .join("train")
        .join("checkpoint_groups")
        .join("group-a")
        .join("checkpoint");
    assert!(
        checkpoint_root.is_dir(),
        "checkpoint root must exist under the checkpoint_group anchor"
    );

    let text = fs::read_to_string(&checkpoint_state_path).expect("read checkpoint state artifact");
    let state: Value = serde_json::from_str(&text).expect("parse checkpoint state artifact");
    assert_eq!(
        state.get("checkpoint_group").and_then(Value::as_str),
        Some("group-a")
    );
    assert_eq!(state.get("resume_epoch").and_then(Value::as_u64), Some(1));
    assert_eq!(
        state.get("checkpoint_root").and_then(Value::as_str),
        Some(checkpoint_root.display().to_string().as_str())
    );

    let observability = fs::read_to_string(artifacts_dir.join("train").join("observability.jsonl"))
        .expect("read train observability");
    assert!(
        observability.contains("\"event\":\"distributed_runtime_checkpoint_state_written\""),
        "train runtime must emit checkpoint-state event"
    );
    assert!(
        observability.contains("\"event\":\"train_done\""),
        "resumed train command must still write train_done to train observability"
    );
    assert!(
        observability.contains("\"event\":\"train_start\""),
        "train_start must stay in the canonical train observability log"
    );
    assert!(
        observability.contains("\"runtime_root\":\"")
            && observability.contains("checkpoint_groups/group-a"),
        "train_start must surface runtime_root when checkpoint_group is active"
    );
    assert!(
        observability.contains("\"checkpoint_group\":\"group-a\""),
        "train_start must surface checkpoint_group in the canonical train log"
    );
}
