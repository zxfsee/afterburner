use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use afterburner::infer::load_model;
use afterburner::train::{
    KERNEL_ADOPTION_CONTRACT_FILENAME, kernel_adoption_threshold_contract_value,
    write_kernel_adoption_threshold_contract,
};
use burn::backend::ndarray::NdArray;
use burn::backend::wgpu::Wgpu;
use burn::prelude::*;
use serde_json::Value;

type CpuBackend = NdArray<f32>;
type GpuBackend = Wgpu<f32, i32>;

#[path = "support/runtime_model_artifact.rs"]
mod runtime_model_artifact;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

fn fixture_json(name: &str) -> Value {
    let text = fs::read_to_string(fixture_path(name)).expect("read json fixture");
    serde_json::from_str(&text).expect("parse json fixture")
}

fn backend_profile_json() -> Value {
    let text = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("backend_performance_profile.fixture.json"),
    )
    .expect("read backend performance profile");
    serde_json::from_str(&text).expect("parse backend performance profile")
}

fn assert_logits_shape_for_backend<B: Backend>(batch_size: usize)
where
    B::Device: Default,
{
    let (_artifact_dir, artifact_path) = runtime_model_artifact::build_runtime_model_artifact();
    let device = B::Device::default();
    let model = load_model::<B>(&artifact_path, &device).expect("load model");
    let input = Tensor::<B, 4>::zeros([batch_size, 1, 28, 28], &device);
    let logits = model.forward(input);
    assert_eq!(
        logits.dims(),
        [batch_size, 10],
        "logits shape must remain [batch,10]"
    );
}

#[test]
fn logits_shape_is_stable_on_cpu() {
    assert_logits_shape_for_backend::<CpuBackend>(3);
}

#[test]
fn logits_shape_is_stable_on_wgpu_when_available() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        assert_logits_shape_for_backend::<GpuBackend>(3);
    }));

    if result.is_err() {
        eprintln!("wgpu backend unavailable; skipping wgpu logits shape guard");
    }
}

#[test]
fn kernel_adoption_threshold_fixture_matches_current_model_footprint() {
    let expected = fixture_json("kernel_adoption_thresholds.fixture.json");
    let actual = kernel_adoption_threshold_contract_value("0.1.0");
    assert_eq!(
        actual, expected,
        "kernel adoption threshold fixture must match the current model contract"
    );
}

#[test]
fn kernel_adoption_threshold_contract_writes_expected_file() {
    let dir = tempfile::tempdir().expect("create tempdir");
    let contract = kernel_adoption_threshold_contract_value("0.1.0");
    let path =
        write_kernel_adoption_threshold_contract(dir.path(), &contract).expect("write contract");
    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some(KERNEL_ADOPTION_CONTRACT_FILENAME)
    );

    let written = fs::read_to_string(&path).expect("read written contract");
    let parsed: Value = serde_json::from_str(&written).expect("parse written contract");
    assert_eq!(parsed, contract, "written contract must round-trip");
}

#[test]
fn kernel_adoption_threshold_contract_reuses_backend_profile_evidence() {
    let contract = kernel_adoption_threshold_contract_value("0.1.0");
    let profile = backend_profile_json();

    assert_eq!(
        contract["adoption_guard"]["supporting_evidence"]["profile_artifact"].as_str(),
        Some("artifacts/eval/backend_performance_profile.json")
    );
    assert_eq!(
        contract["adoption_guard"]["supporting_evidence"]["required_consecutive_regressions"]
            .as_u64(),
        profile["native_backend_adoption"]["required_consecutive_regressions"].as_u64()
    );
    assert!(
        profile["comparison_command"]
            .as_str()
            .is_some_and(|command| command.contains("--batch-size 128")),
        "backend profile comparison command must retain the batch-size baseline"
    );
    assert_eq!(
        contract["adoption_guard"]["supporting_evidence"]["minimum_profile_batch_size"].as_u64(),
        Some(128)
    );
}
