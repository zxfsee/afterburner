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
        "deployment-verification-bundle-rollback-supersession-reconcile supersession current_supersession:",
        "deployment-verification-receipt-transport-locator-reconcile receipt locator:",
        "deployment-verification-handoff-transport-locator-reconcile handoff locator:",
        "scheduler-heartbeat-pointer-rollback-supersession-reconcile supersession current_supersession:",
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
        "deployment-verification-bundle-rollback-manage action +args:",
        "deployment-verification-receipt-manage action +args:",
        "deployment-verification-handoff-manage action +args:",
        "-- debug deploy scheduler-heartbeat-point-rollback-supersede",
        "-- debug deploy record-scheduler-heartbeat-pointer-rollback-history",
        "-- debug deploy scheduler-heartbeat-supersede",
        "scheduler-heartbeat-manage action +args:",
        "scheduler-heartbeat-pointer-rollback-supersession action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "debug deploy recipe bodies must not regress to flat artifact-taxonomy commands `{forbidden}`"
        );
    }
}
