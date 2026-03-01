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

fn finite_row(row: &Value, field: &str) {
    let values = row
        .as_array()
        .unwrap_or_else(|| panic!("{field} row must be an array"));
    assert_eq!(
        values.len(),
        10,
        "{field} row must contain exactly 10 values"
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

    let logits = payload
        .get("logits")
        .and_then(Value::as_array)
        .expect("logits must be an array");
    let probabilities = payload
        .get("probabilities")
        .and_then(Value::as_array)
        .expect("probabilities must be an array");

    assert_eq!(logits.len(), 1, "logits batch size must be stable");
    assert_eq!(
        probabilities.len(),
        1,
        "probabilities batch size must be stable"
    );
    finite_row(&logits[0], "logits");
    finite_row(&probabilities[0], "probabilities");
}
