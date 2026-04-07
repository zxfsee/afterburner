use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use crate::{
    manifest::INPUT_DTYPE,
    model::{MODEL_ARCH_ID, MODEL_ARCH_VERSION, MODEL_KERNEL_SITES},
};

pub const TRAINING_SCALABILITY_CONTRACT_FILENAME: &str = "training_scalability_contract.json";
pub const TRAINING_SCALABILITY_CONTRACT_SCHEMA_VERSION: &str = "1";
pub const KERNEL_ADOPTION_CONTRACT_FILENAME: &str = "kernel_adoption_thresholds.json";
pub const KERNEL_ADOPTION_CONTRACT_SCHEMA_VERSION: &str = "1";
const KERNEL_ADOPTION_MIN_CONV_SITE_COVERAGE_RATIO: f64 = 0.75;
const KERNEL_ADOPTION_MIN_PROFILE_BATCH_SIZE: u64 = 128;
const KERNEL_ADOPTION_REQUIRED_CONSECUTIVE_REGRESSIONS: u64 = 2;
const KERNEL_ADOPTION_PROFILE_ARTIFACT: &str = "artifacts/eval/backend_performance_profile.json";

pub fn training_scalability_contract_value(
    backend: &str,
    artifact_version: &str,
    batch_size: usize,
    worker_parallelism: usize,
    num_epochs: usize,
    samples_per_epoch: u64,
    elapsed_ms: u64,
) -> serde_json::Value {
    let planned_samples = planned_training_samples(samples_per_epoch, num_epochs);
    let throughput_samples_per_sec = (planned_samples as f64 * 1000.0) / elapsed_ms.max(1) as f64;

    serde_json::json!({
        "schema_version": TRAINING_SCALABILITY_CONTRACT_SCHEMA_VERSION,
        "backend": backend,
        "artifact_version": artifact_version,
        "batch_size": batch_size,
        "worker_parallelism": worker_parallelism,
        "num_epochs": num_epochs,
        "samples_per_epoch": samples_per_epoch,
        "planned_samples": planned_samples,
        "elapsed_ms": elapsed_ms,
        "throughput_samples_per_sec": throughput_samples_per_sec
    })
}

pub fn kernel_adoption_threshold_contract_value(artifact_version: &str) -> serde_json::Value {
    let mut kernel_site_counts = BTreeMap::<String, u64>::new();
    let mut padding_modes = BTreeSet::<String>::new();

    for site in MODEL_KERNEL_SITES {
        let key = format!("{}x{}", site.kernel_size[0], site.kernel_size[1]);
        *kernel_site_counts.entry(key).or_insert(0) += 1;
        padding_modes.insert(site.padding.to_string());
    }

    let kernel_sites = MODEL_KERNEL_SITES
        .iter()
        .map(|site| {
            serde_json::json!({
                "name": site.name,
                "kind": site.kind,
                "kernel_size": site.kernel_size,
                "padding": site.padding,
            })
        })
        .collect::<Vec<_>>();

    let kernel_shape_coverage = kernel_site_counts
        .into_iter()
        .map(|(kernel_size, site_count)| {
            serde_json::json!({
                "kernel_size": kernel_size,
                "site_count": site_count,
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "schema_version": KERNEL_ADOPTION_CONTRACT_SCHEMA_VERSION,
        "artifact_version": artifact_version,
        "decision": "defer_custom_kernels",
        "model": {
            "architecture_id": MODEL_ARCH_ID,
            "architecture_version": MODEL_ARCH_VERSION,
            "input_dtype": INPUT_DTYPE,
            "conv_site_count": MODEL_KERNEL_SITES.len(),
            "kernel_shape_coverage": kernel_shape_coverage,
            "kernel_sites": kernel_sites,
        },
        "adoption_guard": {
            "minimum_conv_site_coverage_ratio": KERNEL_ADOPTION_MIN_CONV_SITE_COVERAGE_RATIO,
            "required_padding_modes": padding_modes.into_iter().collect::<Vec<_>>(),
            "forbid_grouped_convolution": true,
            "forbid_dilated_convolution": true,
            "supporting_evidence": {
                "profile_artifact": KERNEL_ADOPTION_PROFILE_ARTIFACT,
                "minimum_profile_batch_size": KERNEL_ADOPTION_MIN_PROFILE_BATCH_SIZE,
                "required_consecutive_regressions": KERNEL_ADOPTION_REQUIRED_CONSECUTIVE_REGRESSIONS,
            }
        }
    })
}

pub fn write_training_scalability_contract(
    train_dir: &Path,
    contract: &serde_json::Value,
) -> io::Result<PathBuf> {
    let path = train_dir.join(TRAINING_SCALABILITY_CONTRACT_FILENAME);
    let json = serde_json::to_string_pretty(contract).map_err(|err| {
        io::Error::other(format!("serialize training scalability contract: {err}"))
    })?;
    std::fs::write(&path, json)?;
    Ok(path)
}

pub fn write_kernel_adoption_threshold_contract(
    train_dir: &Path,
    contract: &serde_json::Value,
) -> io::Result<PathBuf> {
    let path = train_dir.join(KERNEL_ADOPTION_CONTRACT_FILENAME);
    let json = serde_json::to_string_pretty(contract).map_err(|err| {
        io::Error::other(format!(
            "serialize kernel adoption threshold contract: {err}"
        ))
    })?;
    std::fs::write(&path, json)?;
    Ok(path)
}

fn planned_training_samples(samples_per_epoch: u64, num_epochs: usize) -> u64 {
    samples_per_epoch.saturating_mul(num_epochs as u64)
}
