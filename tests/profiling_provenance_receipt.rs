use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn profiling_provenance_receipt_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-030"),
        "architecture decisions index must link ADR-030"
    );
    assert!(
        architecture.contains("profiling provenance receipt"),
        "architecture must mention the profiling provenance receipt contract"
    );
    assert!(
        architecture.contains("profiling_environment_snapshot.json"),
        "architecture must anchor the receipt to the profiling environment snapshot"
    );

    let adr = repo_file("docs/adr/030-profiling-provenance-receipt.md");
    for needle in [
        "profiling_provenance_receipt.json",
        "profiling_environment_snapshot.json",
        "profile_kind",
        "profiler_version",
        "captured_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-030 must mention `{needle}` as part of the profiling provenance receipt contract"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("profiling_provenance_receipt.json"),
        "README must mention the profiling provenance receipt artifact"
    );
}
