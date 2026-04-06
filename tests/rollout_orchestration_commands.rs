use std::fs;
use std::path::PathBuf;

use serde_json::Value;

#[path = "support/runtime_model_artifact.rs"]
mod runtime_model_artifact;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn write_ownership(path: &PathBuf) {
    fs::write(
        path,
        r#"{"schema_version":"1","artifact_version":"0.1.0","artifact_manifest":"artifacts/inference/0.1.0/manifest.toml","provenance_source":"artifacts/train/train_done.jsonl","rollout_owner":"ml-release","approved_by":"ops-review","approved_operations":["upload","deploy"],"approval_ticket":"CHG-4242","approved_at_unix_ms":1735689600000}"#,
    )
    .expect("write ownership");
}

#[test]
fn deploy_rollout_check_and_verify_rollout_write_orchestration_records() {
    let (_artifact_dir, weights_path) = runtime_model_artifact::build_runtime_model_artifact();
    let manifest_path = weights_path
        .parent()
        .expect("artifact dir")
        .join("manifest.toml");
    let tmp = tempfile::tempdir().expect("tempdir");
    let ownership = tmp.path().join("artifact_rollout_ownership.json");
    write_ownership(&ownership);

    let rollout_check = tmp.path().join("rollout_check.json");
    let eval_summary = tmp.path().join("mnist_eval_summary.json");
    let upload_request = tmp.path().join("candidate_upload_request.json");
    let stack_check = tmp.path().join("deployment_stack_check.json");

    let mut check_cmd = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    check_cmd
        .current_dir(repo_root())
        .arg("deploy")
        .arg("rollout-check")
        .arg("--artifact")
        .arg(&weights_path)
        .arg("--manifest")
        .arg(&manifest_path)
        .arg("--ownership")
        .arg(&ownership)
        .arg("--provider")
        .arg("generic")
        .arg("--destination")
        .arg("uploads/afterburner")
        .arg("--repo-root")
        .arg(repo_root())
        .arg("--eval-out")
        .arg(&eval_summary)
        .arg("--upload-out")
        .arg(&upload_request)
        .arg("--stack-check-out")
        .arg(&stack_check)
        .arg("--out-record")
        .arg(&rollout_check)
        .arg("--batch-size")
        .arg("1")
        .arg("--max-batches")
        .arg("1")
        .arg("--min-accuracy")
        .arg("0.0")
        .assert()
        .success();

    let rollout_check_value: Value =
        serde_json::from_str(&fs::read_to_string(&rollout_check).expect("read rollout check"))
            .expect("parse rollout check");
    assert_eq!(
        rollout_check_value["candidate_artifact"],
        Value::from(weights_path.display().to_string())
    );
    assert_eq!(
        rollout_check_value["eval_summary_path"],
        Value::from(eval_summary.display().to_string())
    );
    assert!(
        rollout_check_value["deploy_activate_drv_path"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "rollout check must record the deploy activate drv path"
    );

    let verify_root = tempfile::tempdir().expect("verify tempdir");
    let current_dir = verify_root.path().join("artifacts/inference/0.1.0");
    fs::create_dir_all(&current_dir).expect("create current artifact dir");
    fs::copy(&weights_path, current_dir.join("model.mpk")).expect("copy weights");
    fs::copy(&manifest_path, current_dir.join("manifest.toml")).expect("copy manifest");
    fs::write(
        verify_root.path().join("artifacts/inference/current"),
        "0.1.0\n",
    )
    .expect("write current pointer");

    let rollout_verify = verify_root
        .path()
        .join("artifacts/deploy/rollout_verify.json");
    let verify_eval_summary = verify_root
        .path()
        .join("artifacts/eval/mnist_eval_summary.json");

    let mut verify_cmd = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    verify_cmd
        .current_dir(repo_root())
        .arg("verify")
        .arg("rollout")
        .arg("--artifact")
        .arg(current_dir.join("model.mpk"))
        .arg("--repo-root")
        .arg(verify_root.path())
        .arg("--eval-out")
        .arg(&verify_eval_summary)
        .arg("--out-record")
        .arg(&rollout_verify)
        .arg("--batch-size")
        .arg("1")
        .arg("--max-batches")
        .arg("1")
        .arg("--min-accuracy")
        .arg("0.0")
        .env("BACKEND", "cpu")
        .assert()
        .success();

    let rollout_verify_value: Value =
        serde_json::from_str(&fs::read_to_string(&rollout_verify).expect("read rollout verify"))
            .expect("parse rollout verify");
    assert_eq!(
        rollout_verify_value["candidate_artifact"],
        Value::from(current_dir.join("model.mpk").display().to_string())
    );
    assert_eq!(
        rollout_verify_value["eval_summary_path"],
        Value::from(verify_eval_summary.display().to_string())
    );
    assert!(
        rollout_verify_value["infer_output"].is_object(),
        "rollout verify must capture infer stdout as json"
    );
}
