#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn distributed_runtime_growth_model_and_boundaries_are_documented() {
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

    let adr = repo_file("docs/adr/034-distributed-runtime-growth-model-and-boundaries.md");
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
        "capability-subset work",
        "framework-parity work",
        "DeepSpeed-class systems",
        "worker_parallelism",
        "single-device execution by default",
        "data parallel execution",
        "ZeRO-1/2/3",
        "tensor parallelism",
        "pipeline parallelism",
        "sequence/context parallelism",
        "expert parallelism",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-034 must mention `{needle}` as part of the distributed runtime growth decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "single-device execution by default",
        "single-node DP train path",
        "ZeRO-1/2/3",
        "TP/PP/SP-CP/EP",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must document the current capability surface `{needle}`"
        );
    }
    assert!(
        reference.contains("Scheduler/control plane keeps the lifecycle boundary explicit"),
        "reference index must mention the scheduler/runtime/platform separation"
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
    assert!(
        reference.contains("minimum useful capability subset"),
        "reference index must mention the workload-driven distributed-training subset stance"
    );
}

#[test]
fn distributed_runtime_growth_model_documents_burn_strategy_alignment() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-034"),
        "architecture decisions index must link ADR-034"
    );
    let adr = repo_file("docs/adr/034-distributed-runtime-growth-model-and-boundaries.md");
    for needle in [
        "worker_parallelism",
        "Burn-aligned",
        "data parallel execution",
        "single-node data parallel execution",
        "ZeRO",
        "TP",
        "PP",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-034 must mention `{needle}` as part of the distributed runtime growth decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "worker_parallelism",
        "Distributed runtime remains workload-driven DP-first growth",
        "single-node DP train path",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must document the Burn distributed-learning stance `{needle}`"
        );
    }
}

#[test]
fn cluster_scheduling_strategy_and_adapter_boundary_are_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-056"),
        "architecture decisions index must link ADR-056"
    );
    for needle in [
        "GPU colocation",
        "low-saturation",
        "interference",
        "not the default",
        "kube-rs",
        "node inventory",
        "topology-aware placement",
        "lease ownership",
        "job lifecycle reconciliation",
        "lws",
        "kube-rs-compatible",
        "adapter target",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the cluster scheduling stance"
        );
    }

    let adr = repo_file("docs/adr/056-cluster-scheduling-strategy-and-adapter-boundary.md");
    for needle in [
        "low-saturation",
        "latency regression",
        "throughput regression",
        "memory pressure",
        "default scheduler",
        "kube-rs",
        "node inventory",
        "queue admission",
        "topology-aware GPU placement",
        "lease ownership",
        "job lifecycle reconciliation",
        "single-node scheduler",
        "Do not adopt `lws` now.",
        "kube-rs-compatible",
        "workload-management reference",
        "thin adapter target",
        "subset-of-Kubernetes project",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-056 must mention `{needle}` as part of the cluster scheduling decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("low-saturation workloads"),
        "reference index must mention the constrained colocation stance"
    );
    let readme = repo_file("README.md");
    assert!(
        readme.contains("kube-rs-compatible") && readme.contains("lws"),
        "README must mention the current cluster-path and lws stance"
    );
}
