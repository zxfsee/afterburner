use afterburner::train::{DistributedShardMetadata, validate_distributed_shard_metadata};

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
