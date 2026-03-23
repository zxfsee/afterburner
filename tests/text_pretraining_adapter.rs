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
fn text_token_cache_contract_and_adapter_are_documented() {
    let schema_text =
        fs::read_to_string(fixture_path("text_token_cache.schema.json")).expect("read schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/text-token-cache/v1")
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
        "source",
        "source_revision",
        "tokenizer_id",
        "tokenizer_revision",
        "context_length",
        "vocab_size",
        "train_sequences",
        "validation_sequences",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let run_schema_text = fs::read_to_string(fixture_path("text_pretraining_run.schema.json"))
        .expect("read run schema");
    let run_schema: Value = serde_json::from_str(&run_schema_text).expect("parse run schema json");
    assert_eq!(
        run_schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/text-pretraining-run/v1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-059"),
        "architecture decisions index must link ADR-059"
    );

    let adr = repo_file("docs/adr/059-macbook-text-pretraining-adapter.md");
    for needle in [
        "afterburner train --task text",
        "text_token_cache.schema.json",
        "text_pretraining_run.json",
        "checkpoint/resume",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-059 must mention `{needle}` as part of the text adapter decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("train --task text"),
        "README must mention the text training adapter entrypoint"
    );
}

#[test]
fn train_text_writes_run_artifact_and_text_sidecar() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts_dir = tmp.path().join("artifacts");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.env("BACKEND", "cpu")
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
        .arg("1");
    let assert = cmd.assert().success();

    let run_path = artifacts_dir
        .join("train")
        .join("text_pretraining_run.json");
    let text_inference_dir = artifacts_dir.join("text_inference").join("0.2.0-text");
    let text_inference_profile = text_inference_dir.join("text_inference_profile.json");
    let text_model = text_inference_dir.join("model.mpk");
    let checkpoint_dir = artifacts_dir.join("train").join("text").join("checkpoint");

    assert!(
        run_path.is_file(),
        "text training run artifact must be written"
    );
    assert!(
        text_inference_profile.is_file(),
        "text inference profile sidecar must be written"
    );
    assert!(text_model.is_file(), "text model artifact must be written");
    assert!(checkpoint_dir.is_dir(), "checkpoint directory must exist");

    let run_text = fs::read_to_string(&run_path).expect("read run artifact");
    let run: Value = serde_json::from_str(&run_text).expect("parse run artifact");
    assert_eq!(run.get("task").and_then(Value::as_str), Some("causal-lm"));
    assert_eq!(
        run.get("artifact_version").and_then(Value::as_str),
        Some("0.2.0-text")
    );
    assert_eq!(
        run.get("source").and_then(Value::as_str),
        Some("fineweb-edu/slice")
    );
    assert_eq!(run.get("context_length").and_then(Value::as_u64), Some(8));
    assert_eq!(run.get("vocab_size").and_then(Value::as_u64), Some(32));

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("\"event\":\"text_train_done\""),
        "text training command must emit a text_train_done event"
    );
}
