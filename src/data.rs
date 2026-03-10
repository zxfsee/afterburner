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

fn is_lower_hex_sha256(checksum: &str) -> bool {
    checksum.len() == 64
        && checksum
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
