#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn distributed_runtime_capability_surface_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-041"),
        "architecture decisions index must link ADR-041"
    );
    let adr = repo_file("docs/adr/041-distributed-runtime-capability-surface.md");
    for needle in [
        "single-device execution only",
        "data parallel execution",
        "ZeRO-1/2/3",
        "tensor parallelism",
        "pipeline parallelism",
        "sequence/context parallelism",
        "expert parallelism",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-041 must mention `{needle}` as part of the capability surface decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "single-device execution",
        "First candidate expansion: DP",
        "ZeRO-1/2/3",
        "TP/PP/SP-CP/EP",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must document the current capability surface `{needle}`"
        );
    }
}

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

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("Scheduler/control plane keeps the lifecycle boundary explicit"),
        "reference index must mention the scheduler/runtime/platform separation"
    );
}

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

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("minimum useful capability subset"),
        "reference index must mention the workload-driven distributed-training subset stance"
    );
}

#[test]
fn gpu_colocation_fit_is_documented() {
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
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the colocation stance"
        );
    }

    let adr = repo_file("docs/adr/056-gpu-colocation-fit.md");
    for needle in [
        "low-saturation",
        "latency regression",
        "throughput regression",
        "memory pressure",
        "default scheduler",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-056 must mention `{needle}` as part of the colocation fit decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("low-saturation workloads"),
        "reference index must mention the constrained colocation stance"
    );
}

#[test]
fn kube_rs_gpu_scheduler_placement_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-062"),
        "architecture decisions index must link ADR-062"
    );
    for needle in [
        "kube-rs",
        "node inventory",
        "topology-aware placement",
        "lease ownership",
        "job lifecycle reconciliation",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the kube-rs placement fit"
        );
    }

    let adr = repo_file("docs/adr/062-kube-rs-gpu-scheduler-placement-fit.md");
    for needle in [
        "kube-rs",
        "node inventory",
        "queue admission",
        "topology-aware GPU placement",
        "lease ownership",
        "job lifecycle reconciliation",
        "single-node scheduler",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-062 must mention `{needle}` as part of the cluster-path fit decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("kube-rs-compatible"),
        "README must mention the kube-rs-compatible cluster path"
    );
}

#[test]
fn lws_cluster_workload_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-064"),
        "architecture decisions index must link ADR-064"
    );
    for needle in ["lws", "kube-rs-compatible", "adapter target"] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the lws fit decision"
        );
    }

    let adr = repo_file("docs/adr/064-lws-cluster-workload-fit.md");
    for needle in [
        "Do not adopt `lws` now.",
        "kube-rs-compatible",
        "workload-management reference",
        "thin adapter target",
        "subset-of-Kubernetes project",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-064 must mention `{needle}` as part of the lws fit decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("lws"),
        "README must mention the current lws fit stance"
    );
}
