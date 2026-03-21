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
fn infer_output_drift_baseline_approval_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_approval.schema.json",
    ))
    .expect("read infer drift baseline approval schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline approval");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-approval/v1")
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
        "baseline_path",
        "artifact",
        "artifact_version",
        "policy_profile",
        "approved_by",
        "approval_ticket",
        "approved_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "drift-approve-baseline baseline approved_by approval_ticket approved_at_unix_ms:"
        ),
        "justfile must expose the drift-approve-baseline workflow"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("infer_output_drift_baseline_approval.json"),
        "README must mention the baseline approval artifact"
    );
    assert!(
        readme.contains("just drift-approve-baseline"),
        "README must mention the baseline approval workflow"
    );
}

#[test]
fn drift_approve_baseline_writes_approval_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let baseline = tmp.path().join("baseline.json");
    fs::write(
        &baseline,
        r#"{"schema_version":"1","summary_path":"summary.json","receipt_path":"receipt.json","policy_path":"policy.json","policy_profile":"promotion-default","artifact":"artifacts/inference/candidate/model.mpk","artifact_version":"0.2.0","backend":"cpu","predicted_class":3,"top_probability":0.91,"margin_to_second":0.44,"logits_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","probabilities_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
    )
    .expect("write baseline");

    let out = tmp.path().join("approval.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("approve-baseline")
        .arg("--baseline")
        .arg(&baseline)
        .arg("--approved-by")
        .arg("ops-review")
        .arg("--approval-ticket")
        .arg("CHG-4242")
        .arg("--approved-at-unix-ms")
        .arg("1735689600000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let approval_text = fs::read_to_string(&out).expect("read approval");
    let mut approval: Value = serde_json::from_str(&approval_text).expect("parse approval");
    let object = approval
        .as_object_mut()
        .expect("approval must be a top-level object");
    object.insert("baseline_path".to_string(), Value::from("<baseline>"));
    assert_eq!(
        approval,
        serde_json::json!({
            "schema_version": "1",
            "baseline_path": "<baseline>",
            "artifact": "artifacts/inference/candidate/model.mpk",
            "artifact_version": "0.2.0",
            "policy_profile": "promotion-default",
            "approved_by": "ops-review",
            "approval_ticket": "CHG-4242",
            "approved_at_unix_ms": 1735689600000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_approved");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("approval_path".to_string(), Value::from("<approval>"));
    let approval_obj = fields
        .get_mut("approval")
        .and_then(Value::as_object_mut)
        .expect("event approval must be an object");
    approval_obj.insert("baseline_path".to_string(), Value::from("<baseline>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_approved",
            "fields": {
                "approval_path": "<approval>",
                "approval": approval
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
