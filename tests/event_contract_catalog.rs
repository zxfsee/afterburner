use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use afterburner::model::ModelConfig;
use afterburner::observability::event_line;
use afterburner::train::{
    TRAINING_SCALABILITY_CONTRACT_FILENAME, TrainingConfig, artifact_exported_event_line,
    train_done_event_line, train_start_event_line, training_scalability_contract_value,
    write_training_scalability_contract,
};
use serde_json::{Value, json};

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

fn fixture_json(name: &str) -> Value {
    let text = fs::read_to_string(fixture_path(name)).expect("read fixture");
    serde_json::from_str(&text).expect("parse fixture")
}

fn normalize_train_event_line(line: &str) -> Value {
    let mut value: Value = serde_json::from_str(line).expect("parse event line");
    let object = value
        .as_object_mut()
        .expect("event line must be a top-level object");
    object.insert("ts_ms".to_string(), json!(0));
    value
}

#[test]
fn distributed_tracing_correlation_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-037"),
        "architecture decisions index must link ADR-037"
    );
    for needle in [
        "traceparent",
        "trace_id",
        "AFTERBURNER_TRACEPARENT",
        "Tokio runtime",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the tracing correlation contract"
        );
    }

    let adr = repo_file("docs/adr/037-distributed-tracing-correlation-contract.md");
    for needle in [
        "traceparent",
        "trace_id",
        "AFTERBURNER_TRACEPARENT",
        "Tokio",
        "OpenTelemetry",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-037 must mention `{needle}` as part of the tracing correlation decision"
        );
    }
}

#[test]
fn training_scalability_contract_fixture_matches_written_artifact_and_event() {
    let expected = fixture_json("training_scalability_contract.fixture.json");
    let contract = training_scalability_contract_value("cpu", "0.1.0", 64, 2, 10, 60_000, 1_500);
    assert_eq!(
        contract, expected,
        "training scalability contract artifact must match fixture"
    );

    let tmp = tempfile::tempdir().expect("tempdir");
    let written_path = write_training_scalability_contract(tmp.path(), &contract)
        .expect("write training scalability contract");
    assert_eq!(
        written_path,
        tmp.path().join(TRAINING_SCALABILITY_CONTRACT_FILENAME)
    );
    let written_text =
        fs::read_to_string(&written_path).expect("read written training scalability artifact");
    let written: Value = serde_json::from_str(&written_text).expect("parse written artifact");
    assert_eq!(
        written, expected,
        "written training scalability artifact must match fixture"
    );

    let event: Value =
        serde_json::from_str(&train_done_event_line(&contract)).expect("parse train_done event");
    let mut normalized = event;
    normalized
        .as_object_mut()
        .expect("event must be an object")
        .insert("ts_ms".to_string(), Value::from(0));
    let expected_event = serde_json::json!({
        "ts_ms": 0,
        "level": "info",
        "source": "train",
        "event": "train_done",
        "fields": expected,
    });
    assert_eq!(
        normalized, expected_event,
        "train_done event fields must match training scalability contract fixture"
    );
}

#[test]
fn event_line_matches_required_envelope_fixture_keys() {
    let fixture = fs::read_to_string(fixture_path(
        "observability_envelope_required_keys.fixture.json",
    ))
    .expect("read envelope fixture");
    let fixture: serde_json::Value =
        serde_json::from_str(&fixture).expect("parse envelope fixture");
    let required = fixture
        .get("required")
        .and_then(|v| v.as_array())
        .expect("fixture.required must be an array");
    let required_keys = required
        .iter()
        .map(|v| {
            v.as_str()
                .expect("fixture required keys must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    let line = event_line(
        "info",
        "infer_cli",
        "infer_done",
        json!({ "elapsed_ms": 1 }),
    );
    let event: serde_json::Value = serde_json::from_str(&line).expect("parse event line");
    let obj = event
        .as_object()
        .expect("event line must be a top-level JSON object");

    let actual_keys = obj.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        actual_keys, required_keys,
        "event envelope keys must match fixture"
    );

    assert!(
        event.get("ts_ms").and_then(|v| v.as_u64()).is_some(),
        "ts_ms must be a numeric unix timestamp in milliseconds"
    );
    assert!(
        event.get("fields").and_then(|v| v.as_object()).is_some(),
        "fields must be an object"
    );
}

#[test]
fn train_start_and_artifact_exported_events_match_fixture_contracts() {
    let mut config = TrainingConfig::new(ModelConfig::new(10));
    config.batch_size = 32;
    config.num_workers = 3;
    config.num_epochs = 1;
    config.artifact_version = "0.1.0".to_string();

    let metrics_dir = PathBuf::from("artifacts/train");
    let inference_dir = PathBuf::from("artifacts/inference/0.1.0");

    let line = train_start_event_line(
        "cpu",
        "train_start",
        &config,
        metrics_dir.as_path(),
        metrics_dir.as_path(),
        None,
        inference_dir.as_path(),
        60_000,
    );
    let normalized = normalize_train_event_line(line.as_str());
    let expected_text = fs::read_to_string(fixture_path("train_start_event.fixture.json"))
        .expect("read train_start fixture");
    let expected: Value = serde_json::from_str(&expected_text).expect("parse train_start fixture");
    assert_eq!(
        normalized, expected,
        "train_start event must match the fixture-backed contract"
    );

    let line = artifact_exported_event_line(
        "cpu",
        "0.1.0",
        PathBuf::from("artifacts/inference/0.1.0/model.mpk").as_path(),
        PathBuf::from("artifacts/inference/0.1.0/manifest.toml").as_path(),
        PathBuf::from("artifacts/inference/current").as_path(),
        None,
    );
    let normalized = normalize_train_event_line(line.as_str());
    let expected_text = fs::read_to_string(fixture_path("artifact_exported_event.fixture.json"))
        .expect("read artifact_exported fixture");
    let expected: Value =
        serde_json::from_str(&expected_text).expect("parse artifact_exported fixture");
    assert_eq!(
        normalized, expected,
        "artifact_exported event must match the fixture-backed contract"
    );

    let readme =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/reference.md"))
            .expect("read docs/reference.md");
    assert!(
        readme.contains("artifact_exported"),
        "reference index must mention artifact_exported event fields"
    );
}
