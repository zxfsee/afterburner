use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin_cmd;
use burn::{backend::ndarray::NdArray, prelude::*, record::CompactRecorder};
use serde_json::Value;

type CpuBackend = NdArray<f32>;

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

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn infer_stdout_success_fixture() -> Value {
    let fixture = fs::read_to_string(fixture_path("infer_stdout_success.fixture.json"))
        .expect("read infer fixture");
    serde_json::from_str(&fixture).expect("parse infer fixture")
}

fn finite_row(row: &Value, field: &str, expected_width: usize) {
    let values = row
        .as_array()
        .unwrap_or_else(|| panic!("{field} row must be an array"));
    assert_eq!(
        values.len(),
        expected_width,
        "{field} row must contain exactly {expected_width} values"
    );
    for (idx, value) in values.iter().enumerate() {
        let number = value
            .as_f64()
            .unwrap_or_else(|| panic!("{field}[{idx}] must be numeric"));
        assert!(number.is_finite(), "{field}[{idx}] must be finite");
    }
}

#[test]
fn infer_output_has_finite_logits_probabilities_and_stable_shape() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");
    let weights_path = write_runtime_model_artifact(&artifact_dir);

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.env("BACKEND", "cpu");
    cmd.arg("infer").arg(&weights_path);
    let assert = cmd.assert().success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    let payload: Value =
        serde_json::from_str(stdout.trim()).expect("infer stdout must be a single json object");
    let fixture = infer_stdout_success_fixture();

    let payload_object = payload
        .as_object()
        .expect("infer stdout payload must be an object");
    let fixture_object = fixture
        .as_object()
        .expect("infer stdout fixture must be an object");

    let mut payload_keys = payload_object.keys().cloned().collect::<Vec<_>>();
    let mut fixture_keys = fixture_object.keys().cloned().collect::<Vec<_>>();
    payload_keys.sort();
    fixture_keys.sort();
    assert_eq!(
        payload_keys, fixture_keys,
        "infer stdout top-level keys drifted from fixture"
    );

    for field in fixture_keys {
        let actual = payload_object
            .get(field.as_str())
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("{field} must be an array"));
        let expected = fixture_object
            .get(field.as_str())
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("fixture {field} must be an array"));

        assert_eq!(
            actual.len(),
            expected.len(),
            "{field} batch size must match fixture"
        );
        for (idx, expected_row) in expected.iter().enumerate() {
            let expected_width = expected_row
                .as_array()
                .unwrap_or_else(|| panic!("fixture {field}[{idx}] must be an array"))
                .len();
            finite_row(&actual[idx], field.as_str(), expected_width);
        }
    }

    let logits = payload_object
        .get("logits")
        .and_then(Value::as_array)
        .expect("logits must be present");
    let probabilities = payload_object
        .get("probabilities")
        .and_then(Value::as_array)
        .expect("probabilities must be present");
    assert_eq!(logits.len(), probabilities.len(), "batch size mismatch");
}
