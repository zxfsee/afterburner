#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

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
