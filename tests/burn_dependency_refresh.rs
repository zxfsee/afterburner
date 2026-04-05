mod support;

use support::repo_file;

#[test]
fn burn_dependency_refresh_stance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-033"),
        "architecture decisions index must link ADR-033"
    );
    let adr = repo_file("docs/adr/033-burn-dependency-refresh.md");
    for needle in ["0.20.1", "0.21.0-pre.1", "latest stable", ".mpk", ".bpk"] {
        assert!(
            adr.contains(needle),
            "ADR-033 must mention `{needle}` as part of the refresh decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    for needle in ["Burn dependency refresh", "Burn 0.20.1"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the current Burn refresh stance `{needle}`"
        );
    }
}
