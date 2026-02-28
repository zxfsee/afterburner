use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("inference")
        .join("0.1.0")
        .join(name)
}

#[test]
fn infer_emits_calibration_contract_fields_when_metadata_is_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let weights_path = artifact_dir.join("model.mpk");
    fs::copy(fixture_path("model.mpk"), &weights_path).expect("copy model fixture");
    fs::copy(
        fixture_path("manifest.toml"),
        artifact_dir.join("manifest.toml"),
    )
    .expect("copy manifest fixture");

    fs::write(
        artifact_dir.join("calibration_artifact_metadata.json"),
        r#"{"schema_version":"1","calibration_artifact":"artifacts/calibration/default.calib","artifact_version":"v1","method":"temperature-scaling","created_at_unix_ms":1735689600000}"#,
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

    let weights_path = artifact_dir.join("model.mpk");
    fs::copy(fixture_path("model.mpk"), &weights_path).expect("copy model fixture");
    fs::copy(
        fixture_path("manifest.toml"),
        artifact_dir.join("manifest.toml"),
    )
    .expect("copy manifest fixture");

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
    let invalid_fields = invalid
        .get("fields")
        .and_then(|value| value.as_object())
        .expect("event fields must be an object");
    assert_eq!(
        invalid_fields.get("kind").and_then(|v| v.as_str()),
        Some("parse_error")
    );
    assert!(
        invalid_fields
            .get("detail")
            .and_then(|v| v.as_str())
            .is_some_and(|detail| detail.contains("invalid json")),
        "expected parse error detail"
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
        Some("v1")
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
