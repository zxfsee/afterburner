use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

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

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("low-saturation workloads"),
        "reference index must mention the constrained colocation stance"
    );
}
