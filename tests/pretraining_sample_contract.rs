use afterburner::data::{
    PretrainingSampleMetadataError, PretrainingSplit, parse_pretraining_sample_metadata,
};

#[test]
fn pretraining_sample_metadata_parses_valid_contract() {
    let metadata = serde_json::json!({
        "schema_version": "2",
        "source": "c4/en",
        "source_revision": "2026-03-10-snapshot",
        "sample_id": "train-000001",
        "split": "train",
        "checksum": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });

    let parsed = parse_pretraining_sample_metadata(&metadata).expect("valid metadata should parse");
    assert_eq!(parsed.schema_version, 2);
    assert_eq!(parsed.source, "c4/en");
    assert_eq!(parsed.source_revision, "2026-03-10-snapshot");
    assert_eq!(parsed.sample_id, "train-000001");
    assert_eq!(parsed.split, PretrainingSplit::Train);
    assert_eq!(
        parsed.checksum,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
}

#[test]
fn pretraining_sample_metadata_rejects_invalid_split() {
    let metadata = serde_json::json!({
        "schema_version": "2",
        "source": "c4/en",
        "source_revision": "2026-03-10-snapshot",
        "sample_id": "validation-000001",
        "split": "dev",
        "checksum": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });

    let err = parse_pretraining_sample_metadata(&metadata).expect_err("invalid split must fail");
    assert_eq!(
        err,
        PretrainingSampleMetadataError::InvalidField(
            "split",
            "expected one of: train, validation, test".to_string()
        )
    );
}

#[test]
fn pretraining_sample_metadata_rejects_invalid_checksum() {
    let metadata = serde_json::json!({
        "schema_version": "2",
        "source": "c4/en",
        "source_revision": "2026-03-10-snapshot",
        "sample_id": "validation-000001",
        "split": "validation",
        "checksum": "ABCD"
    });

    let err = parse_pretraining_sample_metadata(&metadata).expect_err("invalid checksum must fail");
    assert_eq!(
        err,
        PretrainingSampleMetadataError::InvalidField(
            "checksum",
            "expected 64 lowercase hex characters".to_string()
        )
    );
}

#[test]
fn pretraining_sample_metadata_rejects_unknown_key() {
    let metadata = serde_json::json!({
        "schema_version": "2",
        "source": "c4/en",
        "source_revision": "2026-03-10-snapshot",
        "sample_id": "train-000001",
        "split": "train",
        "checksum": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "extra": "unexpected"
    });

    let err = parse_pretraining_sample_metadata(&metadata).expect_err("unknown key must fail");
    assert_eq!(
        err,
        PretrainingSampleMetadataError::InvalidField(
            "pretraining_sample_metadata",
            "unknown field: extra".to_string()
        )
    );
}

#[test]
fn pretraining_sample_metadata_rejects_missing_source_revision() {
    let metadata = serde_json::json!({
        "schema_version": "2",
        "source": "c4/en",
        "sample_id": "train-000001",
        "split": "train",
        "checksum": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });

    let err = parse_pretraining_sample_metadata(&metadata)
        .expect_err("missing source_revision must fail");
    assert_eq!(
        err,
        PretrainingSampleMetadataError::MissingField("source_revision")
    );
}
