use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn artifact_retention_envelope_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-019"),
        "architecture decisions index must link ADR-019"
    );
    assert!(
        architecture.contains("artifacts/inference/current"),
        "architecture must mention retaining the current inference pointer"
    );
    assert!(
        architecture.contains("artifacts/profiling/"),
        "architecture must mention profiling artifact retention or pruning"
    );

    let adr = repo_file("docs/adr/019-artifact-retention-envelope.md");
    for needle in [
        "artifacts/inference/current",
        "artifacts/inference/<version>/",
        "artifacts/deploy/",
        "artifacts/train/",
        "artifacts/profiling/",
        "prune",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-019 must mention `{needle}` in the retention envelope"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("retention envelope"),
        "reference index must mention the artifact retention envelope"
    );
    assert!(
        readme.contains("artifacts/profiling/"),
        "reference index must mention the profiling retention policy"
    );
}
