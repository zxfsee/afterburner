use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn text_pretraining_eval_summary_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path("text_pretraining_eval_summary.schema.json"))
        .expect("read text eval schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/text-pretraining-eval-summary/v1")
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("required must be array")
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
        "event",
        "traceparent",
        "trace_id",
        "artifact_version",
        "task",
        "source",
        "source_revision",
        "batch_size",
        "num_epochs",
        "batches_evaluated",
        "validation_sequences",
        "validation_tokens",
        "validation_loss",
        "validation_perplexity",
        "duration_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("text_pretraining_eval_summary.json"),
        "workflow reference must mention the text eval summary artifact"
    );

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("text_pretraining_eval_summary.json"),
        "reference index must mention the text eval summary artifact"
    );
}

#[test]
fn train_text_writes_eval_summary_and_enforces_thresholds() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts_dir = tmp.path().join("artifacts");

    let mut pass_cmd = cargo_bin_cmd!("afterburner");
    pass_cmd
        .env("BACKEND", "cpu")
        .env("ARTIFACTS_DIR", &artifacts_dir)
        .env("ARTIFACT_VERSION", "0.2.0-text")
        .arg("train")
        .arg("--task")
        .arg("text")
        .arg("--dataset-manifest")
        .arg(fixture_path("pretraining_dataset_manifest.example.json"))
        .arg("--tokenizer-profile")
        .arg(fixture_path("text_tokenizer_packing_profile.example.json"))
        .arg("--token-cache")
        .arg(fixture_path("text_token_cache.example.json"))
        .arg("--batch-size")
        .arg("2")
        .arg("--num-epochs")
        .arg("1")
        .arg("--max-validation-loss")
        .arg("4.0")
        .arg("--max-validation-perplexity")
        .arg("80.0");
    let pass_assert = pass_cmd.assert().success();

    let summary_path = artifacts_dir
        .join("eval")
        .join("text_pretraining_eval_summary.json");
    assert!(summary_path.is_file(), "text eval summary must be written");

    let summary_text = fs::read_to_string(&summary_path).expect("read text eval summary");
    let summary: Value = serde_json::from_str(&summary_text).expect("parse text eval summary");
    assert_eq!(
        summary.get("event").and_then(Value::as_str),
        Some("text_pretraining_eval_summary")
    );
    assert_eq!(
        summary.get("task").and_then(Value::as_str),
        Some("causal-lm")
    );
    assert_eq!(
        summary.get("artifact_version").and_then(Value::as_str),
        Some("0.2.0-text")
    );

    let loss = summary
        .get("validation_loss")
        .and_then(Value::as_f64)
        .expect("validation_loss must be numeric");
    let perplexity = summary
        .get("validation_perplexity")
        .and_then(Value::as_f64)
        .expect("validation_perplexity must be numeric");
    assert!(loss <= 4.0, "validation loss must satisfy smoke threshold");
    assert!(
        perplexity <= 80.0,
        "validation perplexity must satisfy smoke threshold"
    );

    let pass_stderr =
        String::from_utf8(pass_assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        pass_stderr.contains("\"event\":\"text_eval_done\""),
        "text training smoke must emit text_eval_done"
    );
    assert!(
        pass_stderr.contains("\"event\":\"text_eval_gate_passed\""),
        "text training smoke must emit text_eval_gate_passed"
    );

    let fail_dir = tmp.path().join("artifacts_fail");
    let mut fail_cmd = cargo_bin_cmd!("afterburner");
    fail_cmd
        .env("BACKEND", "cpu")
        .env("ARTIFACTS_DIR", &fail_dir)
        .env("ARTIFACT_VERSION", "0.2.0-text-fail")
        .arg("train")
        .arg("--task")
        .arg("text")
        .arg("--dataset-manifest")
        .arg(fixture_path("pretraining_dataset_manifest.example.json"))
        .arg("--tokenizer-profile")
        .arg(fixture_path("text_tokenizer_packing_profile.example.json"))
        .arg("--token-cache")
        .arg(fixture_path("text_token_cache.example.json"))
        .arg("--batch-size")
        .arg("2")
        .arg("--num-epochs")
        .arg("1")
        .arg("--max-validation-loss")
        .arg("0.1");
    let fail_assert = fail_cmd.assert().failure();
    let fail_stderr =
        String::from_utf8(fail_assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        fail_stderr.contains("\"event\":\"text_eval_gate_failed\""),
        "threshold failure must emit text_eval_gate_failed"
    );
}
