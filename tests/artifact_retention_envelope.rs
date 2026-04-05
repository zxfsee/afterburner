mod support;

use support::repo_file;

#[test]
fn artifact_retention_envelope_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-019"),
        "architecture decisions index must link ADR-019"
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
    for needle in ["artifacts/inference/current", "artifacts/profiling/"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the retention anchor `{needle}`"
        );
    }
}
