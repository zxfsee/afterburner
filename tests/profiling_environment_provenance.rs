use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn profiling_environment_provenance_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-025"),
        "architecture decisions index must link ADR-025"
    );
    assert!(
        architecture.contains("profiling environment provenance"),
        "architecture must mention the profiling environment provenance contract"
    );
    assert!(
        architecture.contains("profiling_hotspot_summary.schema.json"),
        "architecture must anchor provenance to the profiling summary contract"
    );

    let adr = repo_file("docs/adr/025-profiling-environment-provenance.md");
    for needle in [
        "profiling_hotspot_summary.schema.json",
        "host_os",
        "host_arch",
        "profiler_version",
        "profiler_path",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-025 must mention `{needle}` as part of profiling provenance"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("profiling environment provenance"),
        "README must mention the profiling environment provenance contract"
    );
}
