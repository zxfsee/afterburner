use std::fs;
use std::path::PathBuf;

use afterburner::train::{
    TRAINING_SCALABILITY_CONTRACT_FILENAME, train_done_event_line,
    training_scalability_contract_value, write_training_scalability_contract,
};
use serde_json::Value;

mod support;

use support::fixture_path;

fn fixture_json(name: &str) -> Value {
    let text = fs::read_to_string(fixture_path(name)).expect("read fixture");
    serde_json::from_str(&text).expect("parse fixture")
}

#[test]
fn training_scalability_contract_fixture_matches_written_artifact_and_event() {
    let expected = fixture_json("training_scalability_contract.fixture.json");
    let contract = training_scalability_contract_value("cpu", "0.1.0", 64, 2, 10, 1_500);
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
