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
fn deployment_verification_receipt_contract_is_documented() {
    let schema_text =
        fs::read_to_string(fixture_path("deployment_verification_receipt.schema.json"))
            .expect("read deployment verification receipt schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse deployment verification receipt schema");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-verification-receipt/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-022"),
        "architecture decisions index must link ADR-022"
    );
    assert!(
        architecture.contains("deployment verification receipt"),
        "architecture must mention the deployment verification receipt contract"
    );
    assert!(
        architecture.contains("artifact_version"),
        "architecture must anchor the receipt to artifact_version"
    );

    let adr = repo_file("docs/adr/022-deployment-verification-receipt.md");
    for needle in [
        "deployment_verification_receipt.json",
        "deployment_verification_receipt_written",
        "artifact_version",
        "profile_name",
        "verification_status",
        "verified_at_unix_ms",
        "evidence",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-022 must mention `{needle}` as part of the receipt contract"
        );
    }

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("deployment_verification_receipt.json"),
        "workflow reference must mention the deployment verification receipt artifact"
    );
    assert!(
        readme.contains("rollout-verify"),
        "workflow reference must anchor the receipt to rollout-verify"
    );
    assert!(
        readme.contains("just deployment-verification-receipt"),
        "workflow reference must mention the deployment verification receipt workflow"
    );
}

#[test]
fn deployment_verification_receipt_writes_receipt_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("deployment_verification_receipt.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("verification-receipt")
        .arg("--artifact-version")
        .arg("0.2.0")
        .arg("--profile-name")
        .arg("local-rollout-verify")
        .arg("--verification-status")
        .arg("passed")
        .arg("--verified-at-unix-ms")
        .arg("1735689600000")
        .arg("--evidence")
        .arg("local rollout verify")
        .arg("--evidence-source")
        .arg("artifacts/eval/mnist_eval_summary.json,eval_done,1735689601000")
        .arg("--evidence-source")
        .arg("artifacts/deploy/candidate_upload_request.json,artifact_upload_request_written,1735689602000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read deployment verification receipt");
    let receipt: Value =
        serde_json::from_str(&text).expect("parse deployment verification receipt");
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "artifact_version": "0.2.0",
            "profile_name": "local-rollout-verify",
            "verification_status": "passed",
            "verified_at_unix_ms": 1735689600000_u64,
            "evidence": "local rollout verify",
            "evidence_sources": [
                {
                    "artifact_path": "artifacts/eval/mnist_eval_summary.json",
                    "event_name": "eval_done",
                    "observed_at_unix_ms": 1735689601000_u64
                },
                {
                    "artifact_path": "artifacts/deploy/candidate_upload_request.json",
                    "event_name": "artifact_upload_request_written",
                    "observed_at_unix_ms": 1735689602000_u64
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
                == Some("deployment_verification_receipt_written")
        })
        .unwrap_or_else(|| {
            panic!("missing deployment verification receipt event in stderr: {stderr}")
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
            "source": "deploy_cli",
            "event": "deployment_verification_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}
