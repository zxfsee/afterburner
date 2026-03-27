use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
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
