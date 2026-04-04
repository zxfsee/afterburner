use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn profiling_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "profile-infer:",
        "profile-environment-snapshot:",
        "profile-refresh-environment-snapshot current_snapshot profiler_path captured_at_unix_ms:",
        "profile-provenance-receipt snapshot captured_at_unix_ms:",
        "profile-provenance-bundle snapshot summary captured_at_unix_ms:",
        "workflow-surface-check-profiling:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Profiling stays grouped under these operator flows:",
        "Hotspot capture and infer path: `just profile-infer` delegates to `afterburner profile infer`.",
        "Environment capture: `just profile-environment-snapshot`, `just profile-refresh-environment-snapshot`.",
        "Provenance and evidence: `just profile-provenance-receipt`, `just profile-provenance-bundle`.",
        "On macOS, profiling requires `xcrun xctrace version` under full Xcode.",
        "The native profiling command forces `XCTRACE=/usr/bin/xctrace` while clearing `DEVELOPER_DIR` and `SDKROOT`.",
        "`just workflow-surface-check-profiling` guards the grouped profiling workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped profiling surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just profile-environment-snapshot` writes `profiling_environment_snapshot.json`.",
        "- `just profile-refresh-environment-snapshot` writes `profiling_environment_snapshot_refresh.json`.",
        "- `just profile-provenance-receipt` writes `profiling_provenance_receipt.json`.",
        "- `just profile-provenance-bundle` writes `profiling_provenance_evidence_bundle.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating profiling variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Profiling:",
        "`infer_hotspot_summary.json`",
        "`profiling_environment_snapshot.json`",
        "`profiling_environment_snapshot_refresh.json`",
        "`profiling_provenance_receipt.json`",
        "`profiling_provenance_evidence_bundle.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-profiling` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped profiling surface `{needle}`"
        );
    }
}
