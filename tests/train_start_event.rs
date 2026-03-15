use std::fs;
use std::path::PathBuf;

use afterburner::model::ModelConfig;
use afterburner::train::{TrainingConfig, train_start_event_line};
use serde_json::{Value, json};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn train_start_event_matches_fixture_contract() {
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
        inference_dir.as_path(),
    );
    let normalized = normalize_train_start_event_line(line.as_str());
    let expected_text = fs::read_to_string(fixture_path("train_start_event.json"))
        .expect("read train_start fixture");
    let expected: Value = serde_json::from_str(&expected_text).expect("parse train_start fixture");
    assert_eq!(
        normalized, expected,
        "train_start event must match the fixture-backed contract"
    );
}

fn normalize_train_start_event_line(line: &str) -> Value {
    let mut value: Value = serde_json::from_str(line).expect("parse train_start event");
    let object = value
        .as_object_mut()
        .expect("train_start event must be a top-level object");
    object.insert("ts_ms".to_string(), json!(0));
    value
}
