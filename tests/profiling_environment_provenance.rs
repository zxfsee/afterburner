mod support;

use support::repo_file;

#[test]
fn profiling_environment_provenance_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-025"),
        "architecture decisions index must link ADR-025"
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

    let readme = repo_file("docs/reference.md");
    for needle in [
        "profiling environment provenance",
        "profiling_hotspot_summary.schema.json",
    ] {
        assert!(
            readme.contains(needle),
            "reference index must mention the profiling environment provenance anchor `{needle}`"
        );
    }
}
