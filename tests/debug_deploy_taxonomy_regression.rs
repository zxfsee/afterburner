use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn debug_deploy_surface_stays_family_first() {
    let justfile = repo_file("justfile");
    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");

    for required in [
        "-- debug deploy verification-bundle record-rollback-history",
        "-- debug deploy verification-receipt point-locator",
        "-- debug deploy verification-handoff reconcile",
        "scheduler-heartbeat-manage action +args:",
        "fn regrouped_debug_deploy_subcommand(",
        "\"verification-bundle\"",
        "\"verification-receipt\"",
        "\"verification-handoff\"",
        "\"scheduler-heartbeat\"",
    ] {
        assert!(
            justfile.contains(required) || deploy_dispatch.contains(required),
            "debug deploy taxonomy must keep grouped family-first surface `{required}`"
        );
    }

    for forbidden in [
        "-- debug deploy record-verification-bundle-rollback-history",
        "-- debug deploy record-verification-receipt-locator-history",
        "-- debug deploy record-verification-handoff-history",
        "-- debug deploy scheduler-heartbeat-point-rollback-supersede",
        "-- debug deploy record-scheduler-heartbeat-pointer-rollback-history",
        "-- debug deploy scheduler-heartbeat-supersede",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "debug deploy recipe bodies must not regress to flat artifact-taxonomy commands `{forbidden}`"
        );
    }
}
