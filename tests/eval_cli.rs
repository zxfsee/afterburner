use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::collections::BTreeSet;
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
    let eval_done = stderr_event(&eval_stderr, "eval_done");
    let eval_version = event_field_string(&eval_done, "artifact_version");

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

#[test]
fn eval_emits_monitoring_contract_event_and_summary_fields() {
    let artifact = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("inference")
        .join("0.1.0")
        .join("model.mpk");
    let tmp = tempfile::tempdir().expect("tempdir");
    let summary_path = tmp.path().join("eval_summary.json");

    let mut eval_cmd = cargo_bin_cmd!("afterburner");
    eval_cmd
        .arg("eval")
        .arg(&artifact)
        .arg("--seed")
        .arg("42")
        .arg("--batch-size")
        .arg("16")
        .arg("--max-batches")
        .arg("1")
        .arg("--out")
        .arg(&summary_path);
    let eval_assert = eval_cmd.assert().success();
    let eval_stderr =
        String::from_utf8(eval_assert.get_output().stderr.clone()).expect("eval stderr utf8");
    let eval_done = stderr_event(&eval_stderr, "eval_done");
    let normalized_event = normalize_eval_done_event(&eval_done);
    let expected_event = fixture_json("eval_pipeline_monitoring_event.json");
    assert_eq!(
        normalized_event, expected_event,
        "eval_done event must match the monitoring contract fixture"
    );

    let summary_text = fs::read_to_string(&summary_path).expect("read eval summary");
    let summary: Value = serde_json::from_str(&summary_text).expect("parse eval summary");
    assert_eq!(
        summary.get("event").and_then(Value::as_str),
        Some("mnist_eval_summary")
    );
    assert_eq!(
        summary.get("artifact_version").and_then(Value::as_str),
        Some("0.1.0")
    );
    assert_eq!(summary.get("seed").and_then(Value::as_u64), Some(42));
    assert_eq!(summary.get("batch_size").and_then(Value::as_u64), Some(16));
    assert_eq!(
        summary.get("batches_evaluated").and_then(Value::as_u64),
        Some(1)
    );

    let samples = summary
        .get("samples")
        .and_then(Value::as_u64)
        .expect("summary samples must be numeric");
    assert!(samples > 0, "summary samples must be positive");

    let accuracy = summary
        .get("accuracy")
        .and_then(Value::as_f64)
        .expect("summary accuracy must be numeric");
    assert!(
        (0.0..=1.0).contains(&accuracy),
        "summary accuracy must be normalized"
    );

    assert_eq!(
        summary.get("error_count").and_then(Value::as_u64),
        Some(0),
        "successful eval summary must report zero errors"
    );
    assert!(
        summary.get("duration_ms").and_then(Value::as_u64).is_some(),
        "summary duration_ms must be numeric"
    );
    assert!(
        summary
            .get("rate_samples_per_sec")
            .and_then(Value::as_f64)
            .is_some(),
        "summary rate_samples_per_sec must be numeric"
    );
}

#[test]
fn eval_summary_schema_fixture_has_required_monitoring_fields() {
    let schema = fixture_json("eval_pipeline_monitoring_artifact.schema.json");

    assert_eq!(
        schema.get("$schema").and_then(Value::as_str),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/eval-pipeline-monitoring-artifact/v1")
    );
    assert_eq!(schema.get("type").and_then(Value::as_str), Some("object"));
    assert_eq!(
        schema.get("additionalProperties").and_then(Value::as_bool),
        Some(false)
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
        "event",
        "artifact",
        "artifact_version",
        "seed",
        "batch_size",
        "max_batches",
        "batches_evaluated",
        "samples",
        "correct",
        "accuracy",
        "error_count",
        "duration_ms",
        "rate_samples_per_sec",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .expect("schema.properties must be an object");
    assert_eq!(
        properties
            .get("event")
            .and_then(|value| value.get("const"))
            .and_then(Value::as_str),
        Some("mnist_eval_summary")
    );
    assert_eq!(
        properties
            .get("artifact")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("string")
    );
    assert_eq!(
        properties
            .get("artifact_version")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("string")
    );
    assert_eq!(
        properties
            .get("accuracy")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("number")
    );
    assert_eq!(
        properties
            .get("error_count")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("integer")
    );
    assert_eq!(
        properties
            .get("duration_ms")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("integer")
    );
    assert_eq!(
        properties
            .get("rate_samples_per_sec")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("number")
    );
}

#[test]
fn eval_checked_in_summary_matches_monitoring_schema_contract() {
    let schema = fixture_json("eval_pipeline_monitoring_artifact.schema.json");
    let summary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("eval")
        .join("mnist_eval_summary.json");
    let summary_text = fs::read_to_string(summary_path).expect("read eval summary sample artifact");
    let summary: Value = serde_json::from_str(&summary_text).expect("parse eval summary json");
    let summary = summary
        .as_object()
        .expect("eval summary sample must be a top-level object");

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
    let actual = summary.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(actual, required, "summary keys must match schema.required");

    assert_eq!(
        summary.get("event").and_then(Value::as_str),
        Some("mnist_eval_summary")
    );
    for key in ["artifact", "artifact_version"] {
        let value = summary
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{key} must be a string"));
        assert!(!value.is_empty(), "{key} must be non-empty");
    }
    for key in [
        "seed",
        "batch_size",
        "max_batches",
        "batches_evaluated",
        "samples",
        "correct",
        "error_count",
        "duration_ms",
    ] {
        summary
            .get(key)
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("{key} must be an integer"));
    }

    let accuracy = summary
        .get("accuracy")
        .and_then(Value::as_f64)
        .expect("accuracy must be numeric");
    assert!(
        (0.0..=1.0).contains(&accuracy),
        "accuracy must be normalized"
    );

    let rate = summary
        .get("rate_samples_per_sec")
        .and_then(Value::as_f64)
        .expect("rate_samples_per_sec must be numeric");
    assert!(rate >= 0.0, "rate_samples_per_sec must be non-negative");
}

#[test]
fn eval_summary_success_fixture_pins_top_level_contract() {
    let fixture = fixture_json("eval_summary_success.json");
    let summary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("eval")
        .join("mnist_eval_summary.json");
    let summary_text = fs::read_to_string(summary_path).expect("read eval summary sample artifact");
    let summary: Value = serde_json::from_str(&summary_text).expect("parse eval summary json");

    let fixture_object = fixture
        .as_object()
        .expect("eval summary success fixture must be an object");
    let summary_object = summary
        .as_object()
        .expect("eval summary sample must be an object");

    let fixture_keys = fixture_object.keys().cloned().collect::<BTreeSet<_>>();
    let summary_keys = summary_object.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        summary_keys, fixture_keys,
        "eval summary top-level keys must match fixture contract"
    );

    for key in ["event", "artifact_version"] {
        assert_eq!(
            summary_object.get(key),
            fixture_object.get(key),
            "{key} must match fixture contract"
        );
    }
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

fn fixture_json(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    let text = fs::read_to_string(path).expect("read fixture");
    serde_json::from_str(&text).expect("parse fixture")
}

fn normalize_eval_done_event(event: &Value) -> Value {
    let mut normalized = event.clone();
    let object = normalized
        .as_object_mut()
        .expect("event must be a top-level object");
    object.insert("ts_ms".to_string(), Value::from(0));

    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");

    fields.insert(
        "artifact".to_string(),
        Value::from("<artifact>/artifacts/inference/0.1.0/model.mpk"),
    );

    let duration_ms = fields
        .get("duration_ms")
        .and_then(Value::as_u64)
        .expect("duration_ms must be numeric");
    assert!(duration_ms >= 1, "duration_ms must be positive");
    fields.insert("duration_ms".to_string(), Value::from(1));

    let rate = fields
        .get("rate_samples_per_sec")
        .and_then(Value::as_f64)
        .expect("rate_samples_per_sec must be numeric");
    assert!(rate > 0.0, "rate_samples_per_sec must be positive");
    fields.insert("rate_samples_per_sec".to_string(), Value::from(1.0));

    normalized
}
