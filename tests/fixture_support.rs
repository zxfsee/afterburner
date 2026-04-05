#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use afterburner::manifest::{ArtifactManifest, compute_sha256_hex};
use burn::{backend::ndarray::NdArray, prelude::*, record::CompactRecorder};
use png::{ColorType, Decoder};

pub struct SummaryStats {
    pub mean: f32,
    pub std: f32,
    pub min: f32,
    pub max: f32,
}

type CpuBackend = NdArray<f32>;

pub fn load_fixture_image() -> [[f32; 28]; 28] {
    let path = fixture_path("mnist_0.png");
    let file = fs::File::open(path).expect("open fixture png");
    let decoder = Decoder::new(file);
    let mut reader = decoder.read_info().expect("read png info");
    let info = reader.info();
    assert_eq!(info.width, 28, "fixture width mismatch");
    assert_eq!(info.height, 28, "fixture height mismatch");
    assert_eq!(info.color_type, ColorType::Grayscale);
    assert_eq!(info.bit_depth, png::BitDepth::Eight);

    let mut buf = vec![0u8; reader.output_buffer_size()];
    let output = reader.next_frame(&mut buf).expect("decode png");
    let bytes = &buf[..output.buffer_size()];

    let mut image = [[0.0f32; 28]; 28];
    for y in 0..28 {
        for x in 0..28 {
            image[y][x] = bytes[y * 28 + x] as f32;
        }
    }
    image
}

pub fn load_fixture_summary() -> SummaryStats {
    let path = fixture_path("mnist_0.summary.toml");
    let contents = fs::read_to_string(path).expect("read summary");
    let value: toml::Value = toml::from_str(&contents).expect("parse summary");

    SummaryStats {
        mean: read_f32(&value, "mean"),
        std: read_f32(&value, "std"),
        min: read_f32(&value, "min"),
        max: read_f32(&value, "max"),
    }
}

pub fn load_manifest_fixture() -> String {
    let path = fixture_path("manifest.toml");
    fs::read_to_string(path).expect("read manifest template")
}

pub fn fixture_model_path() -> PathBuf {
    fixture_path("model.mpk")
}

pub fn fixture_manifest_path() -> PathBuf {
    fixture_path("manifest.toml")
}

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

mod support;

use support::fixture_path;

fn read_f32(value: &toml::Value, field: &str) -> f32 {
    value
        .get(field)
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|v| v as f64)))
        .map(|v| v as f32)
        .unwrap_or_else(|| panic!("missing {field}"))
}
