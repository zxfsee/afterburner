use std::fs;

use afterburner::train::{
    DistributedShardMetadata, distributed_shard_metadata_invalid_event_line,
    validate_distributed_shard_metadata, validate_distributed_shard_metadata_load_path,
};

fn valid_metadata_fixture() -> Vec<DistributedShardMetadata> {
    vec![
        DistributedShardMetadata {
            shard_index: 0,
            shard_count: 4,
            owner_worker_id: "worker-00".to_string(),
            owner_worker_rank: 0,
        },
        DistributedShardMetadata {
            shard_index: 1,
            shard_count: 4,
            owner_worker_id: "worker-01".to_string(),
            owner_worker_rank: 1,
        },
        DistributedShardMetadata {
            shard_index: 2,
            shard_count: 4,
            owner_worker_id: "worker-02".to_string(),
            owner_worker_rank: 2,
        },
    ]
}

#[test]
fn distributed_shard_validator_accepts_unique_owner_rank_and_bounded_indices() {
    let metadata = valid_metadata_fixture();
    assert!(validate_distributed_shard_metadata(&metadata).is_ok());
}

#[test]
fn distributed_shard_validator_rejects_duplicate_owner_rank_tuple() {
    let mut metadata = valid_metadata_fixture();
    metadata[2].owner_worker_id = "worker-01".to_string();
    metadata[2].owner_worker_rank = 1;

    let err = validate_distributed_shard_metadata(&metadata);
    assert_eq!(
        err,
        Err(
            "metadata[2] duplicates owner_worker_id 'worker-01' and owner_worker_rank 1"
                .to_string()
        )
    );
}

#[test]
fn distributed_shard_validator_rejects_shard_index_out_of_bounds() {
    let mut metadata = valid_metadata_fixture();
    metadata[1].shard_index = 4;

    let err = validate_distributed_shard_metadata(&metadata);
    assert_eq!(
        err,
        Err("metadata[1] has shard_index 4 not less than shard_count 4".to_string())
    );
}

#[test]
fn distributed_shard_validator_load_path_reports_structured_failure_for_invalid_index() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let metadata_path = tmp.path().join("distributed_shards.json");
    fs::write(
        &metadata_path,
        r#"[{"shard_index":0,"shard_count":2,"owner_worker_id":"worker-00","owner_worker_rank":0},{"shard_index":2,"shard_count":2,"owner_worker_id":"worker-01","owner_worker_rank":1}]"#,
    )
    .expect("write distributed shard metadata fixture");

    let err = validate_distributed_shard_metadata_load_path(&metadata_path)
        .expect_err("invalid shard metadata must fail");
    assert_eq!(err.kind(), "validation_error");
    assert_eq!(err.metadata_path(), metadata_path.as_path());
    assert_eq!(
        err.detail(),
        "metadata[1] has shard_index 2 not less than shard_count 2"
    );
}

#[test]
fn distributed_shard_validator_load_path_error_maps_to_structured_event() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let metadata_path = tmp.path().join("distributed_shards.json");
    fs::write(
        &metadata_path,
        r#"[{"shard_index":1,"shard_count":1,"owner_worker_id":"worker-00","owner_worker_rank":0}]"#,
    )
    .expect("write distributed shard metadata fixture");

    let err = validate_distributed_shard_metadata_load_path(&metadata_path)
        .expect_err("invalid shard metadata must fail");
    let line = distributed_shard_metadata_invalid_event_line("cpu", "0.1.0", &err);
    let event: serde_json::Value = serde_json::from_str(line.as_str()).expect("parse event line");

    assert_eq!(event.get("level").and_then(|v| v.as_str()), Some("error"));
    assert_eq!(event.get("source").and_then(|v| v.as_str()), Some("train"));
    assert_eq!(
        event.get("event").and_then(|v| v.as_str()),
        Some("distributed_shard_metadata_invalid")
    );

    let fields = event
        .get("fields")
        .and_then(|value| value.as_object())
        .expect("event fields must be object");
    assert_eq!(fields.get("backend").and_then(|v| v.as_str()), Some("cpu"));
    assert_eq!(
        fields.get("artifact_version").and_then(|v| v.as_str()),
        Some("0.1.0")
    );
    assert_eq!(
        fields.get("kind").and_then(|v| v.as_str()),
        Some("validation_error")
    );
    assert_eq!(
        fields.get("metadata_path").and_then(|v| v.as_str()),
        Some(metadata_path.to_string_lossy().as_ref())
    );
    assert_eq!(
        fields.get("detail").and_then(|v| v.as_str()),
        Some("metadata[0] has shard_index 1 not less than shard_count 1")
    );
}
