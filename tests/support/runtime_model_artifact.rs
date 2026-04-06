use std::path::PathBuf;

use afterburner::manifest::{ArtifactManifest, compute_sha256_hex};
use burn::{backend::ndarray::NdArray, prelude::*, record::CompactRecorder};

type CpuBackend = NdArray<f32>;

pub fn build_runtime_model_artifact() -> (tempfile::TempDir, PathBuf) {
    let artifact_dir = tempfile::tempdir().expect("create model artifact tempdir");
    let weights_path = artifact_dir.path().join("model.mpk");

    let device = <CpuBackend as Backend>::Device::default();
    let model = afterburner::model::ModelConfig::new(10).init::<CpuBackend>(&device);
    model
        .save_file(&weights_path, &CompactRecorder::new())
        .expect("write model artifact");

    let checksum = compute_sha256_hex(&weights_path).expect("compute model checksum");
    let manifest = ArtifactManifest::for_current("model.mpk", "0.1.0", checksum);
    manifest
        .write_to_dir(artifact_dir.path())
        .expect("write model manifest");

    (artifact_dir, weights_path)
}
