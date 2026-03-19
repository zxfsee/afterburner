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
fn infer_output_drift_baseline_handoff_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_handoff.schema.json",
    ))
    .expect("read infer drift baseline handoff schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline handoff");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-handoff/v1")
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
        "bundle_path",
        "approval_path",
        "baseline_path",
        "artifact",
        "artifact_version",
        "policy_profile",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("drift-export-baseline-handoff bundle:"),
        "justfile must expose the drift-export-baseline-handoff workflow"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("infer_output_drift_baseline_handoff.json"),
        "README must mention the baseline handoff manifest"
    );
    assert!(
        readme.contains("just drift-export-baseline-handoff"),
        "README must mention the baseline handoff workflow"
    );
}

#[test]
fn drift_export_baseline_handoff_writes_manifest_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp.path().join("bundle.json");
    fs::write(
        &bundle,
        r#"{"schema_version":"1","pointer_path":"pointer.json","history_path":"history.json","approval_path":"approval.json","baseline_path":"baseline.json","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default","history_entry_count":1}"#,
    )
    .expect("write bundle");

    let out = tmp.path().join("handoff.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift-export-baseline-handoff")
        .arg("--bundle")
        .arg(&bundle)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read handoff");
    let mut handoff: Value = serde_json::from_str(&text).expect("parse handoff");
    handoff
        .as_object_mut()
        .expect("handoff must be object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        handoff,
        serde_json::json!({
            "schema_version": "1",
            "bundle_path": "<bundle>",
            "approval_path": "approval.json",
            "baseline_path": "baseline.json",
            "artifact": "artifacts/inference/0.2.0/model.mpk",
            "artifact_version": "0.2.0",
            "policy_profile": "promotion-default"
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_handoff_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("handoff_path".to_string(), Value::from("<handoff>"));
    let handoff_obj = fields
        .get_mut("handoff")
        .and_then(Value::as_object_mut)
        .expect("event handoff must be object");
    handoff_obj.insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_handoff_written",
            "fields": {
                "handoff_path": "<handoff>",
                "handoff": handoff
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
