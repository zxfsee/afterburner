use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn burn_distributed_learning_strategy_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-038"),
        "architecture decisions index must link ADR-038"
    );
    assert!(
        architecture.contains("worker_parallelism"),
        "architecture must mention the current worker_parallelism stance"
    );
    assert!(
        architecture.contains("not a distributed strategy contract"),
        "architecture must state that the current scalability contract is not a distributed strategy contract"
    );

    let adr = repo_file("docs/adr/038-burn-distributed-learning-strategy-fit.md");
    for needle in [
        "worker_parallelism",
        "TrainingStrategy",
        "with_training_strategy",
        "MultiDevicesTrainStep",
        "optimize_multi",
        "world-size",
        "device-group",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-038 must mention `{needle}` as part of the Burn learning-strategy fit decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("`worker_parallelism` is only a"),
        "README must document the local-only worker_parallelism stance"
    );
}
