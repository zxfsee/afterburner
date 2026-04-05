use afterburner::data::{
    PretrainingDatasetManifestError, PretrainingSplit, parse_pretraining_dataset_manifest,
};
use std::fs;

mod support;

use support::fixture_path;

#[test]
fn pretraining_dataset_manifest_parses_valid_contract() {
    let text = fs::read_to_string(fixture_path("pretraining_dataset_manifest.example.json"))
        .expect("read dataset manifest fixture");
    let value: serde_json::Value =
        serde_json::from_str(&text).expect("parse dataset manifest fixture");

    let parsed = parse_pretraining_dataset_manifest(&value).expect("valid dataset manifest");
    assert_eq!(parsed.schema_version, 1);
    assert_eq!(parsed.source, "fineweb-edu/slice");
    assert_eq!(parsed.source_revision, "2026-03-22-macbook-100m-tokens");
    assert_eq!(parsed.shard_count, 3);
    assert_eq!(parsed.total_samples, 1500);
    assert_eq!(parsed.split_sample_counts.train, 1200);
    assert_eq!(parsed.split_sample_counts.validation, 200);
    assert_eq!(parsed.split_sample_counts.test, 100);
    assert_eq!(
        parsed.checksum_rollup_sha256,
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
    );
    assert_eq!(parsed.shards.len(), 3);
    assert_eq!(parsed.shards[0].split, PretrainingSplit::Train);
    assert_eq!(parsed.shards[1].split, PretrainingSplit::Validation);
    assert_eq!(parsed.shards[2].split, PretrainingSplit::Test);
}

#[test]
fn pretraining_dataset_manifest_rejects_split_count_mismatch() {
    let value = serde_json::json!({
        "schema_version": "1",
        "source": "fineweb-edu/slice",
        "source_revision": "2026-03-22-macbook-100m-tokens",
        "shard_count": 1,
        "total_samples": 100,
        "split_sample_counts": {
            "train": 99,
            "validation": 0,
            "test": 0
        },
        "checksum_rollup_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "shards": [
            {
                "shard_id": "train-0000",
                "split": "train",
                "sample_count": 100,
                "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }
        ]
    });

    let err =
        parse_pretraining_dataset_manifest(&value).expect_err("split count mismatch must fail");
    assert_eq!(
        err,
        PretrainingDatasetManifestError::InvalidField(
            "split_sample_counts.train",
            "expected 100 samples from shard inventory".to_string()
        )
    );
}

#[test]
fn pretraining_dataset_manifest_rejects_duplicate_shard_id() {
    let value = serde_json::json!({
        "schema_version": "1",
        "source": "fineweb-edu/slice",
        "source_revision": "2026-03-22-macbook-100m-tokens",
        "shard_count": 2,
        "total_samples": 2,
        "split_sample_counts": {
            "train": 2,
            "validation": 0,
            "test": 0
        },
        "checksum_rollup_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "shards": [
            {
                "shard_id": "train-0000",
                "split": "train",
                "sample_count": 1,
                "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            },
            {
                "shard_id": "train-0000",
                "split": "train",
                "sample_count": 1,
                "checksum_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            }
        ]
    });

    let err = parse_pretraining_dataset_manifest(&value).expect_err("duplicate shard_id must fail");
    assert_eq!(
        err,
        PretrainingDatasetManifestError::InvalidField(
            "shards",
            "duplicate shard_id: train-0000".to_string()
        )
    );
}
