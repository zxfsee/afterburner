mod support;

use support::repo_file;

#[test]
fn distributed_optimizer_and_checkpoint_state_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-061"),
        "architecture decisions index must link ADR-061"
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
    let reference = repo_file("docs/reference.md");
    for needle in ["distributed optimizer-state recovery", "checkpoint_group"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the optimizer/checkpoint anchor `{needle}`"
        );
    }
}
