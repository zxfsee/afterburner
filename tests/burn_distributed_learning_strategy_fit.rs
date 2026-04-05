use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn burn_distributed_learning_strategy_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-038"),
        "architecture decisions index must link ADR-038"
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

    let readme = repo_file("docs/reference.md");
    for needle in ["worker_parallelism", "not a distributed strategy contract"] {
        assert!(
            readme.contains(needle),
            "reference index must document the Burn distributed-learning stance `{needle}`"
        );
    }
}
