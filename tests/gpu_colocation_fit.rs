use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
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

    let readme = repo_file("README.md");
    assert!(
        readme.contains("low-saturation workloads"),
        "README must mention the constrained colocation stance"
    );
}
