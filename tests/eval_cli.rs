use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn eval_fails_fast_on_missing_artifact() {
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("eval").arg("does-not-exist.mpk");
    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"eval_error""#)
            .and(predicate::str::contains("does-not-exist.mpk")),
    );
}

#[test]
fn eval_accepts_inline_artifact_flag() {
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("eval").arg("--artifact=does-not-exist-inline.mpk");
    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"eval_error""#)
            .and(predicate::str::contains("does-not-exist-inline.mpk"))
            .and(predicate::str::contains("unknown argument").not()),
    );
}

#[test]
fn eval_default_artifact_path_uses_current_version_pointer() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let inference_dir = tmp.path().join("artifacts").join("inference");
    fs::create_dir_all(&inference_dir).expect("create inference dir");
    fs::write(inference_dir.join("current"), "9.9.9\n").expect("write current pointer");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.current_dir(tmp.path());
    cmd.arg("eval");
    cmd.assert()
        .failure()
        .code(2)
        .stderr(
            predicate::str::contains(r#""event":"eval_error""#).and(predicate::str::contains(
                "artifacts/inference/9.9.9/model.mpk",
            )),
        );
}

#[test]
fn eval_emits_artifact_version_and_matches_infer_resolution() {
    let artifact = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("inference")
        .join("0.1.0")
        .join("model.mpk");
    let tmp = tempfile::tempdir().expect("tempdir");
    let summary_path = tmp.path().join("eval_summary.json");

    let mut infer_cmd = cargo_bin_cmd!("afterburner");
    infer_cmd.env("BACKEND", "cpu");
    infer_cmd.arg("infer").arg(&artifact);
    let infer_assert = infer_cmd.assert().success();
    let infer_stderr = String::from_utf8(infer_assert.get_output().stderr.clone())
        .expect("infer stderr must be utf8");
    let infer_done = stderr_event(&infer_stderr, "infer_done");
    let infer_version = event_field_string(&infer_done, "artifact_version");

    let mut eval_cmd = cargo_bin_cmd!("afterburner");
    eval_cmd
        .arg("eval")
        .arg(&artifact)
        .arg("--batch-size")
        .arg("16")
        .arg("--max-batches")
        .arg("1")
        .arg("--out")
        .arg(&summary_path);
    let eval_assert = eval_cmd.assert().success();
    let eval_stderr =
        String::from_utf8(eval_assert.get_output().stderr.clone()).expect("eval stderr utf8");
    let eval_written = stderr_event(&eval_stderr, "mnist_eval_written");
    let eval_version = event_field_string(&eval_written, "artifact_version");

    assert_eq!(
        eval_version, infer_version,
        "eval artifact_version must match infer contract resolution"
    );

    let summary_text = fs::read_to_string(&summary_path).expect("read eval summary");
    let summary: Value = serde_json::from_str(&summary_text).expect("parse eval summary");
    let summary_version = summary
        .get("artifact_version")
        .and_then(Value::as_str)
        .expect("summary artifact_version must be present");
    assert_eq!(
        summary_version, infer_version,
        "summary artifact_version must match infer contract resolution"
    );
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}

fn event_field_string(event: &Value, field: &str) -> String {
    event
        .get("fields")
        .and_then(Value::as_object)
        .and_then(|fields| fields.get(field))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("missing string field `{field}` in event: {event}"))
}
