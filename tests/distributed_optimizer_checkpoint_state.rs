use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn distributed_optimizer_and_checkpoint_state_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-061"),
        "architecture decisions index must link ADR-061"
    );
    assert!(
        architecture.contains("optimizer-state"),
        "architecture must mention the distributed optimizer-state contract"
    );
    assert!(
        architecture.contains("checkpoint_group"),
        "architecture must anchor the contract to checkpoint_group"
    );

    let adr = repo_file("docs/adr/061-distributed-optimizer-and-checkpoint-state.md");
    for needle in [
        "optimizer-state",
        "checkpoint_group",
        "scheduler",
        "stop/resume",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-061 must mention `{needle}` as part of the recovery decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("checkpoint_group"),
        "README must mention the explicit checkpoint_group recovery anchor"
    );
}
