#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn distributed_runtime_scheduler_platform_separation_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-034"),
        "architecture decisions index must link ADR-034"
    );
    for needle in [
        "Distributed Training Layers",
        "Distributed training runtime",
        "GPU scheduler / allocator",
        "Platform / infrastructure substrate",
        "START",
        "CHECKPOINTED",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the 3-layer boundary"
        );
    }

    let adr = repo_file("docs/adr/034-distributed-runtime-scheduler-platform-separation.md");
    for needle in [
        "distributed training runtime",
        "GPU scheduler / allocator",
        "platform / infrastructure substrate",
        "`START`",
        "`STOP`",
        "`KILL`",
        "`READY`",
        "`CHECKPOINTED`",
        "`FAILED`",
        "`HEARTBEAT`",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-034 must mention `{needle}` as part of the layer separation decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("decides who gets GPUs and when"),
        "reference index must mention the scheduler/runtime/platform separation"
    );
}
