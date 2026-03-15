use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin_cmd;
use burn::{backend::ndarray::NdArray, prelude::*, record::CompactRecorder};
use serde_json::json;

type CpuBackend = NdArray<f32>;

fn event_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn event_fixture(name: &str) -> serde_json::Value {
    let text = fs::read_to_string(event_fixture_path(name)).expect("read event fixture");
    serde_json::from_str(&text).expect("parse event fixture")
}

fn write_runtime_model_artifact(artifact_dir: &Path) -> PathBuf {
    let weights_path = artifact_dir.join("model.mpk");

    let device = <CpuBackend as Backend>::Device::default();
    let model = afterburner::model::ModelConfig::new(10).init::<CpuBackend>(&device);
    model
        .save_file(&weights_path, &CompactRecorder::new())
        .expect("write model artifact");

    let checksum =
        afterburner::manifest::compute_sha256_hex(&weights_path).expect("compute model checksum");
    let manifest =
        afterburner::manifest::ArtifactManifest::for_current("model.mpk", "0.1.0", checksum);
    manifest
        .write_to_dir(artifact_dir)
        .expect("write model manifest");

    weights_path
}

#[test]
fn infer_emits_calibration_contract_fields_when_metadata_is_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let weights_path = write_runtime_model_artifact(&artifact_dir);

    fs::write(
        artifact_dir.join("calibration_artifact_metadata.json"),
        r#"{"schema_version":"1","calibration_artifact":"artifacts/calibration/default.calib","artifact_version":"0.1.0","method":"temperature-scaling","created_at_unix_ms":1735689600000}"#,
    )
    .expect("write calibration metadata");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer")
        .arg("--artifact")
        .arg(&weights_path)
        .env("BACKEND", "cpu");
    let assert = cmd.assert().success().code(0);

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let events: Vec<serde_json::Value> = stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("parse event line"))
        .collect();

    assert_calibration_fields(find_event(&events, "artifact_load_ok"));
    assert_calibration_fields(find_event(&events, "infer_done"));
}

#[test]
fn infer_emits_calibration_metadata_invalid_event_on_parse_failure() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let weights_path = write_runtime_model_artifact(&artifact_dir);

    fs::write(
        artifact_dir.join("calibration_artifact_metadata.json"),
        r#"{"schema_version":"1","calibration_artifact":"x""#,
    )
    .expect("write invalid calibration metadata");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer")
        .arg("--artifact")
        .arg(&weights_path)
        .env("BACKEND", "cpu");
    let assert = cmd.assert().success().code(0);

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let events: Vec<serde_json::Value> = stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("parse event line"))
        .collect();

    let invalid = find_event(&events, "calibration_metadata_invalid");
    let normalized = normalize_calibration_metadata_invalid_event(invalid);
    let expected = event_fixture("infer_calibration_metadata_invalid_event.json");
    assert_eq!(
        normalized, expected,
        "calibration_metadata_invalid event payload must match fixture"
    );

    assert_no_calibration_fields(find_event(&events, "artifact_load_ok"));
    assert_no_calibration_fields(find_event(&events, "infer_done"));
}

#[test]
fn infer_omits_calibration_when_metadata_targets_different_artifact_version() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let weights_path = write_runtime_model_artifact(&artifact_dir);

    fs::write(
        artifact_dir.join("calibration_artifact_metadata.json"),
        r#"{"schema_version":"1","calibration_artifact":"artifacts/calibration/default.calib","artifact_version":"9.9.9","method":"temperature-scaling","created_at_unix_ms":1735689600000}"#,
    )
    .expect("write mismatched calibration metadata");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer")
        .arg("--artifact")
        .arg(&weights_path)
        .env("BACKEND", "cpu");
    let assert = cmd.assert().success().code(0);

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let events: Vec<serde_json::Value> = stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("parse event line"))
        .collect();

    let invalid = find_event(&events, "calibration_metadata_invalid");
    let normalized = normalize_calibration_metadata_invalid_event(invalid);
    let expected = event_fixture("infer_calibration_version_mismatch_event.json");
    assert_eq!(
        normalized, expected,
        "artifact-version mismatch event payload must match fixture"
    );

    assert_no_calibration_fields(find_event(&events, "artifact_load_ok"));
    assert_no_calibration_fields(find_event(&events, "infer_done"));
}

fn find_event<'a>(events: &'a [serde_json::Value], event: &str) -> &'a serde_json::Value {
    events
        .iter()
        .find(|item| item.get("event").and_then(|v| v.as_str()) == Some(event))
        .unwrap_or_else(|| panic!("missing event {event}"))
}

fn assert_calibration_fields(event: &serde_json::Value) {
    let fields = event
        .get("fields")
        .and_then(|value| value.as_object())
        .expect("event fields must be an object");
    let calibration = fields
        .get("calibration")
        .and_then(|value| value.as_object())
        .expect("calibration fields must be present");

    assert_eq!(
        calibration.get("schema_version").and_then(|v| v.as_str()),
        Some("1")
    );
    assert_eq!(
        calibration
            .get("calibration_artifact")
            .and_then(|v| v.as_str()),
        Some("artifacts/calibration/default.calib")
    );
    assert_eq!(
        calibration.get("artifact_version").and_then(|v| v.as_str()),
        Some("0.1.0")
    );
    assert_eq!(
        calibration.get("method").and_then(|v| v.as_str()),
        Some("temperature-scaling")
    );
    assert_eq!(
        calibration
            .get("created_at_unix_ms")
            .and_then(|v| v.as_u64()),
        Some(1_735_689_600_000)
    );
}

fn assert_no_calibration_fields(event: &serde_json::Value) {
    let fields = event
        .get("fields")
        .and_then(|value| value.as_object())
        .expect("event fields must be an object");
    assert!(
        !fields.contains_key("calibration"),
        "calibration field must be absent"
    );
}

fn normalize_calibration_metadata_invalid_event(event: &serde_json::Value) -> serde_json::Value {
    let mut normalized = event.clone();
    let object = normalized
        .as_object_mut()
        .expect("event must be represented as an object");

    object.insert("ts_ms".to_string(), json!(0));

    let fields = object
        .get_mut("fields")
        .and_then(|value| value.as_object_mut())
        .expect("event fields must be an object");

    fields.insert(
        "metadata".to_string(),
        json!("<artifact>/calibration_artifact_metadata.json"),
    );

    normalized
}
