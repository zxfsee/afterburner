#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn distributed_training_capability_subset_scope_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-035"),
        "architecture decisions index must link ADR-035"
    );
    for needle in [
        "capability-subset work",
        "DeepSpeed-class systems",
        "pattern",
        "parity targets",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the distributed-training scope rule"
        );
    }

    let adr = repo_file("docs/adr/035-distributed-training-capability-subset-scope.md");
    for needle in [
        "capability-subset work",
        "framework-parity work",
        "DeepSpeed-class systems",
        "Phase 1",
        "Phase 2",
        "Phase 3",
        "DP",
        "ZeRO-1",
        "TP",
        "PP",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-035 must mention `{needle}` as part of the subset-scope decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("minimum useful capability subset"),
        "reference index must mention the workload-driven distributed-training subset stance"
    );
}
