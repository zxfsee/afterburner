use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn burn_dependency_refresh_stance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-033"),
        "architecture decisions index must link ADR-033"
    );
    assert!(
        architecture.contains("Burn dependency refresh"),
        "architecture must mention the Burn dependency refresh decision"
    );
    assert!(
        architecture.contains("0.20.1"),
        "architecture must anchor the current Burn pin"
    );

    let adr = repo_file("docs/adr/033-burn-dependency-refresh.md");
    for needle in ["0.20.1", "0.21.0-pre.1", "latest stable", ".mpk", ".bpk"] {
        assert!(
            adr.contains(needle),
            "ADR-033 must mention `{needle}` as part of the refresh decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("Burn 0.20.1"),
        "reference index must mention the current Burn refresh stance"
    );
}
