use std::collections::BTreeSet;
use std::sync::Arc;

use burn::{
    data::dataloader::batcher::Batcher,
    data::dataset::vision::{MnistDataset, MnistItem},
    prelude::*,
};
use serde_json::{Map, Value};

use crate::preprocess::mnist_image_to_tensor;

pub const PRETRAINING_SAMPLE_METADATA_SCHEMA_VERSION: &str = "2";
pub const PRETRAINING_SAMPLE_METADATA_REQUIRED_FIELDS: [&str; 6] = [
    "schema_version",
    "source",
    "source_revision",
    "sample_id",
    "split",
    "checksum",
];
pub const PRETRAINING_SAMPLE_METADATA_SPLIT_VALUES: [&str; 3] = ["train", "validation", "test"];
pub const PRETRAINING_SAMPLE_METADATA_CHECKSUM_PATTERN: &str = "^[a-f0-9]{64}$";
pub const PRETRAINING_DATASET_MANIFEST_SCHEMA_VERSION: &str = "1";
pub const PRETRAINING_DATASET_MANIFEST_REQUIRED_FIELDS: [&str; 8] = [
    "schema_version",
    "source",
    "source_revision",
    "shard_count",
    "total_samples",
    "split_sample_counts",
    "checksum_rollup_sha256",
    "shards",
];

/// Batched MNIST tensors ready for a convolutional classifier.
#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 4>,
    pub targets: Tensor<B, 1, Int>,
}

#[derive(Clone, Default)]
pub struct MnistBatcher;

impl<B: Backend> Batcher<B, MnistItem, MnistBatch<B>> for MnistBatcher {
    fn batch(&self, items: Vec<MnistItem>, device: &B::Device) -> MnistBatch<B> {
        let images = items
            .iter()
            .map(|item| training_preprocess_mnist_image::<B>(item.image, device))
            .collect();

        let images = Tensor::stack(images, 0);
        let targets = items
            .iter()
            .map(|item| Tensor::<B, 1, Int>::from_ints([item.label as i64], device))
            .collect::<Vec<_>>();
        let targets = Tensor::cat(targets, 0);

        MnistBatch { images, targets }
    }
}

pub fn training_preprocess_mnist_image<B: Backend>(
    image: [[f32; 28]; 28],
    device: &B::Device,
) -> Tensor<B, 3> {
    mnist_image_to_tensor::<B>(image, device)
}

pub fn train_loader<B: Backend>(
    batch_size: usize,
    num_workers: usize,
    seed: u64,
    device: B::Device,
) -> Arc<dyn burn::data::dataloader::DataLoader<B, MnistBatch<B>>> {
    burn::data::dataloader::DataLoaderBuilder::new(MnistBatcher)
        .batch_size(batch_size)
        .shuffle(seed)
        .num_workers(num_workers)
        .set_device(device)
        .build(MnistDataset::train())
}

pub fn test_loader<B: Backend>(
    batch_size: usize,
    num_workers: usize,
    device: B::Device,
) -> Arc<dyn burn::data::dataloader::DataLoader<B, MnistBatch<B>>> {
    burn::data::dataloader::DataLoaderBuilder::new(MnistBatcher)
        .batch_size(batch_size)
        .num_workers(num_workers)
        .set_device(device)
        .build(MnistDataset::test())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PretrainingSplit {
    Train,
    Validation,
    Test,
}

impl PretrainingSplit {
    fn parse(value: &str) -> Option<Self> {
        if value == PRETRAINING_SAMPLE_METADATA_SPLIT_VALUES[0] {
            Some(Self::Train)
        } else if value == PRETRAINING_SAMPLE_METADATA_SPLIT_VALUES[1] {
            Some(Self::Validation)
        } else if value == PRETRAINING_SAMPLE_METADATA_SPLIT_VALUES[2] {
            Some(Self::Test)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PretrainingSampleMetadata {
    pub schema_version: u8,
    pub source: String,
    pub source_revision: String,
    pub sample_id: String,
    pub split: PretrainingSplit,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PretrainingSampleMetadataError {
    MissingField(&'static str),
    InvalidField(&'static str, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PretrainingSplitSampleCounts {
    pub train: u64,
    pub validation: u64,
    pub test: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PretrainingDatasetShard {
    pub shard_id: String,
    pub split: PretrainingSplit,
    pub sample_count: u64,
    pub checksum_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PretrainingDatasetManifest {
    pub schema_version: u8,
    pub source: String,
    pub source_revision: String,
    pub shard_count: u64,
    pub total_samples: u64,
    pub split_sample_counts: PretrainingSplitSampleCounts,
    pub checksum_rollup_sha256: String,
    pub shards: Vec<PretrainingDatasetShard>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PretrainingDatasetManifestError {
    MissingField(&'static str),
    InvalidField(&'static str, String),
}

pub fn parse_pretraining_sample_metadata(
    value: &Value,
) -> Result<PretrainingSampleMetadata, PretrainingSampleMetadataError> {
    let object = value
        .as_object()
        .ok_or(PretrainingSampleMetadataError::InvalidField(
            "pretraining_sample_metadata",
            "expected object".to_string(),
        ))?;

    for key in object.keys() {
        if !PRETRAINING_SAMPLE_METADATA_REQUIRED_FIELDS.contains(&key.as_str()) {
            return Err(PretrainingSampleMetadataError::InvalidField(
                "pretraining_sample_metadata",
                format!("unknown field: {key}"),
            ));
        }
    }

    let schema_version = read_required_string(object, "schema_version")?;
    if schema_version != PRETRAINING_SAMPLE_METADATA_SCHEMA_VERSION {
        return Err(PretrainingSampleMetadataError::InvalidField(
            "schema_version",
            format!("expected \"{PRETRAINING_SAMPLE_METADATA_SCHEMA_VERSION}\""),
        ));
    }

    let source = read_required_string(object, "source")?;
    if source.is_empty() {
        return Err(PretrainingSampleMetadataError::InvalidField(
            "source",
            "must be non-empty".to_string(),
        ));
    }

    let source_revision = read_required_string(object, "source_revision")?;
    if source_revision.is_empty() {
        return Err(PretrainingSampleMetadataError::InvalidField(
            "source_revision",
            "must be non-empty".to_string(),
        ));
    }

    let sample_id = read_required_string(object, "sample_id")?;
    if sample_id.is_empty() {
        return Err(PretrainingSampleMetadataError::InvalidField(
            "sample_id",
            "must be non-empty".to_string(),
        ));
    }

    let split = read_required_string(object, "split")?;
    let split =
        PretrainingSplit::parse(split).ok_or(PretrainingSampleMetadataError::InvalidField(
            "split",
            "expected one of: train, validation, test".to_string(),
        ))?;

    let checksum = read_required_string(object, "checksum")?;
    if !is_lower_hex_sha256(checksum) {
        return Err(PretrainingSampleMetadataError::InvalidField(
            "checksum",
            "expected 64 lowercase hex characters".to_string(),
        ));
    }

    Ok(PretrainingSampleMetadata {
        schema_version: 2,
        source: source.to_string(),
        source_revision: source_revision.to_string(),
        sample_id: sample_id.to_string(),
        split,
        checksum: checksum.to_string(),
    })
}

pub fn parse_pretraining_dataset_manifest(
    value: &Value,
) -> Result<PretrainingDatasetManifest, PretrainingDatasetManifestError> {
    let object = value
        .as_object()
        .ok_or(PretrainingDatasetManifestError::InvalidField(
            "pretraining_dataset_manifest",
            "expected object".to_string(),
        ))?;

    for key in object.keys() {
        if !PRETRAINING_DATASET_MANIFEST_REQUIRED_FIELDS.contains(&key.as_str()) {
            return Err(PretrainingDatasetManifestError::InvalidField(
                "pretraining_dataset_manifest",
                format!("unknown field: {key}"),
            ));
        }
    }

    let schema_version = read_required_string_dataset(object, "schema_version")?;
    if schema_version != PRETRAINING_DATASET_MANIFEST_SCHEMA_VERSION {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "schema_version",
            format!("expected \"{PRETRAINING_DATASET_MANIFEST_SCHEMA_VERSION}\""),
        ));
    }

    let source = read_required_string_dataset(object, "source")?;
    if source.is_empty() {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "source",
            "must be non-empty".to_string(),
        ));
    }

    let source_revision = read_required_string_dataset(object, "source_revision")?;
    if source_revision.is_empty() {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "source_revision",
            "must be non-empty".to_string(),
        ));
    }

    let shard_count = read_required_u64_dataset(object, "shard_count")?;
    if shard_count == 0 {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "shard_count",
            "must be greater than 0".to_string(),
        ));
    }

    let total_samples = read_required_u64_dataset(object, "total_samples")?;
    if total_samples == 0 {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "total_samples",
            "must be greater than 0".to_string(),
        ));
    }

    let split_counts = parse_split_sample_counts(object.get("split_sample_counts"))?;

    let checksum_rollup_sha256 = read_required_string_dataset(object, "checksum_rollup_sha256")?;
    if !is_lower_hex_sha256(checksum_rollup_sha256) {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "checksum_rollup_sha256",
            "expected 64 lowercase hex characters".to_string(),
        ));
    }

    let shards = object
        .get("shards")
        .and_then(Value::as_array)
        .ok_or(PretrainingDatasetManifestError::MissingField("shards"))?;
    if shards.is_empty() {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "shards",
            "must include at least one shard".to_string(),
        ));
    }

    let mut parsed_shards = Vec::with_capacity(shards.len());
    let mut shard_ids = BTreeSet::new();
    let mut train_count = 0_u64;
    let mut validation_count = 0_u64;
    let mut test_count = 0_u64;
    let mut computed_total = 0_u64;

    for (index, shard) in shards.iter().enumerate() {
        let shard = shard
            .as_object()
            .ok_or(PretrainingDatasetManifestError::InvalidField(
                "shards",
                format!("shards[{index}] must be an object"),
            ))?;

        for key in shard.keys() {
            if !["shard_id", "split", "sample_count", "checksum_sha256"].contains(&key.as_str()) {
                return Err(PretrainingDatasetManifestError::InvalidField(
                    "shards",
                    format!("shards[{index}] unknown field: {key}"),
                ));
            }
        }

        let shard_id =
            read_required_string_dataset(shard, "shard_id").map_err(|err| match err {
                PretrainingDatasetManifestError::MissingField(_) => {
                    PretrainingDatasetManifestError::MissingField("shards.shard_id")
                }
                PretrainingDatasetManifestError::InvalidField(_, detail) => {
                    PretrainingDatasetManifestError::InvalidField("shards.shard_id", detail)
                }
            })?;
        if shard_id.is_empty() {
            return Err(PretrainingDatasetManifestError::InvalidField(
                "shards.shard_id",
                "must be non-empty".to_string(),
            ));
        }
        if !shard_ids.insert(shard_id.to_string()) {
            return Err(PretrainingDatasetManifestError::InvalidField(
                "shards",
                format!("duplicate shard_id: {shard_id}"),
            ));
        }

        let split = read_required_string_dataset(shard, "split").map_err(|err| match err {
            PretrainingDatasetManifestError::MissingField(_) => {
                PretrainingDatasetManifestError::MissingField("shards.split")
            }
            PretrainingDatasetManifestError::InvalidField(_, detail) => {
                PretrainingDatasetManifestError::InvalidField("shards.split", detail)
            }
        })?;
        let split =
            PretrainingSplit::parse(split).ok_or(PretrainingDatasetManifestError::InvalidField(
                "shards.split",
                "expected one of: train, validation, test".to_string(),
            ))?;

        let sample_count =
            read_required_u64_dataset(shard, "sample_count").map_err(|err| match err {
                PretrainingDatasetManifestError::MissingField(_) => {
                    PretrainingDatasetManifestError::MissingField("shards.sample_count")
                }
                PretrainingDatasetManifestError::InvalidField(_, detail) => {
                    PretrainingDatasetManifestError::InvalidField("shards.sample_count", detail)
                }
            })?;
        if sample_count == 0 {
            return Err(PretrainingDatasetManifestError::InvalidField(
                "shards.sample_count",
                "must be greater than 0".to_string(),
            ));
        }

        let checksum_sha256 =
            read_required_string_dataset(shard, "checksum_sha256").map_err(|err| match err {
                PretrainingDatasetManifestError::MissingField(_) => {
                    PretrainingDatasetManifestError::MissingField("shards.checksum_sha256")
                }
                PretrainingDatasetManifestError::InvalidField(_, detail) => {
                    PretrainingDatasetManifestError::InvalidField("shards.checksum_sha256", detail)
                }
            })?;
        if !is_lower_hex_sha256(checksum_sha256) {
            return Err(PretrainingDatasetManifestError::InvalidField(
                "shards.checksum_sha256",
                "expected 64 lowercase hex characters".to_string(),
            ));
        }

        computed_total += sample_count;
        match split {
            PretrainingSplit::Train => train_count += sample_count,
            PretrainingSplit::Validation => validation_count += sample_count,
            PretrainingSplit::Test => test_count += sample_count,
        }

        parsed_shards.push(PretrainingDatasetShard {
            shard_id: shard_id.to_string(),
            split,
            sample_count,
            checksum_sha256: checksum_sha256.to_string(),
        });
    }

    if shard_count != parsed_shards.len() as u64 {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "shard_count",
            format!("expected {} shard entries", parsed_shards.len()),
        ));
    }

    if total_samples != computed_total {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "total_samples",
            format!("expected {computed_total} samples from shard inventory"),
        ));
    }

    if split_counts.train != train_count {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "split_sample_counts.train",
            format!("expected {train_count} samples from shard inventory"),
        ));
    }
    if split_counts.validation != validation_count {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "split_sample_counts.validation",
            format!("expected {validation_count} samples from shard inventory"),
        ));
    }
    if split_counts.test != test_count {
        return Err(PretrainingDatasetManifestError::InvalidField(
            "split_sample_counts.test",
            format!("expected {test_count} samples from shard inventory"),
        ));
    }

    Ok(PretrainingDatasetManifest {
        schema_version: 1,
        source: source.to_string(),
        source_revision: source_revision.to_string(),
        shard_count,
        total_samples,
        split_sample_counts: split_counts,
        checksum_rollup_sha256: checksum_rollup_sha256.to_string(),
        shards: parsed_shards,
    })
}

fn read_required_string<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a str, PretrainingSampleMetadataError> {
    match object.get(field) {
        Some(value) => value.as_str().ok_or_else(|| {
            PretrainingSampleMetadataError::InvalidField(field, "expected string".to_string())
        }),
        None => Err(PretrainingSampleMetadataError::MissingField(field)),
    }
}

fn read_required_string_dataset<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a str, PretrainingDatasetManifestError> {
    match object.get(field) {
        Some(value) => value.as_str().ok_or_else(|| {
            PretrainingDatasetManifestError::InvalidField(field, "expected string".to_string())
        }),
        None => Err(PretrainingDatasetManifestError::MissingField(field)),
    }
}

fn read_required_u64_dataset(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<u64, PretrainingDatasetManifestError> {
    match object.get(field) {
        Some(value) => value.as_u64().ok_or_else(|| {
            PretrainingDatasetManifestError::InvalidField(
                field,
                "expected non-negative integer".to_string(),
            )
        }),
        None => Err(PretrainingDatasetManifestError::MissingField(field)),
    }
}

fn parse_split_sample_counts(
    value: Option<&Value>,
) -> Result<PretrainingSplitSampleCounts, PretrainingDatasetManifestError> {
    let object =
        value
            .and_then(Value::as_object)
            .ok_or(PretrainingDatasetManifestError::MissingField(
                "split_sample_counts",
            ))?;

    for key in object.keys() {
        if !["train", "validation", "test"].contains(&key.as_str()) {
            return Err(PretrainingDatasetManifestError::InvalidField(
                "split_sample_counts",
                format!("unknown field: {key}"),
            ));
        }
    }

    Ok(PretrainingSplitSampleCounts {
        train: read_required_u64_dataset(object, "train").map_err(|err| match err {
            PretrainingDatasetManifestError::MissingField(_) => {
                PretrainingDatasetManifestError::MissingField("split_sample_counts.train")
            }
            PretrainingDatasetManifestError::InvalidField(_, detail) => {
                PretrainingDatasetManifestError::InvalidField("split_sample_counts.train", detail)
            }
        })?,
        validation: read_required_u64_dataset(object, "validation").map_err(|err| match err {
            PretrainingDatasetManifestError::MissingField(_) => {
                PretrainingDatasetManifestError::MissingField("split_sample_counts.validation")
            }
            PretrainingDatasetManifestError::InvalidField(_, detail) => {
                PretrainingDatasetManifestError::InvalidField(
                    "split_sample_counts.validation",
                    detail,
                )
            }
        })?,
        test: read_required_u64_dataset(object, "test").map_err(|err| match err {
            PretrainingDatasetManifestError::MissingField(_) => {
                PretrainingDatasetManifestError::MissingField("split_sample_counts.test")
            }
            PretrainingDatasetManifestError::InvalidField(_, detail) => {
                PretrainingDatasetManifestError::InvalidField("split_sample_counts.test", detail)
            }
        })?,
    })
}

fn is_lower_hex_sha256(checksum: &str) -> bool {
    checksum.len() == 64
        && checksum
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
