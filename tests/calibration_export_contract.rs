use std::fs;
use std::path::PathBuf;

use afterburner::train::{
    artifact_exported_event_line, copy_calibration_metadata_sidecar_if_present,
};
use serde_json::{Value, json};

mod support;

use support::fixture_path;

#[test]
fn export_copies_calibration_sidecar_and_emits_calibration_fields() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let train_dir = tmp.path().join("train");
    let inference_dir = tmp.path().join("inference").join("0.1.0");
    fs::create_dir_all(&train_dir).expect("create train dir");
    fs::create_dir_all(&inference_dir).expect("create inference dir");

    fs::write(
        train_dir.join("calibration_artifact_metadata.json"),
        r#"{"schema_version":"1","calibration_artifact":"artifacts/calibration/default.calib","artifact_version":"0.1.0","method":"temperature-scaling","created_at_unix_ms":1735689600000}"#,
    )
    .expect("write calibration metadata");

    let copied = copy_calibration_metadata_sidecar_if_present(&train_dir, &inference_dir, "0.1.0")
        .expect("copy calibration sidecar")
        .expect("calibration sidecar should be copied");
    assert_eq!(
        copied.path,
        inference_dir.join("calibration_artifact_metadata.json"),
        "copied calibration sidecar must land beside the exported artifact"
    );

    let copied_text = fs::read_to_string(&copied.path).expect("read copied calibration sidecar");
    let copied_json: Value = serde_json::from_str(&copied_text).expect("parse copied calibration");
    let expected_json = json!({
        "schema_version":"1",
        "calibration_artifact":"artifacts/calibration/default.calib",
        "artifact_version":"0.1.0",
        "method":"temperature-scaling",
        "created_at_unix_ms":1735689600000_u64
    });
    assert_eq!(copied_json, expected_json);

    let line = artifact_exported_event_line(
        "cpu",
        "0.1.0",
        PathBuf::from("artifacts/inference/0.1.0/model.mpk").as_path(),
        PathBuf::from("artifacts/inference/0.1.0/manifest.toml").as_path(),
        PathBuf::from("artifacts/inference/current").as_path(),
        Some(&copied),
    );
    let mut actual: Value = serde_json::from_str(&line).expect("parse artifact_exported event");
    let object = actual.as_object_mut().expect("event must be an object");
    object.insert("ts_ms".to_string(), json!(0));
    object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object")
        .insert(
            "calibration_metadata_path".to_string(),
            json!("artifacts/inference/0.1.0/calibration_artifact_metadata.json"),
        );
    let expected_text = fs::read_to_string(fixture_path(
        "artifact_exported_event_with_calibration.fixture.json",
    ))
    .expect("read calibration artifact_exported fixture");
    let expected: Value =
        serde_json::from_str(&expected_text).expect("parse calibration artifact_exported fixture");
    assert_eq!(
        actual, expected,
        "artifact_exported must surface copied calibration metadata explicitly"
    );
}
