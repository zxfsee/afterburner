mod support;

use support::repo_file;

#[test]
fn debug_deploy_surface_stays_family_first() {
    let justfile = repo_file("justfile");
    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");

    for required in [
        "fn regrouped_debug_deploy_subcommand(",
        "\"verification-bundle\"",
        "\"verification-receipt\"",
        "\"verification-handoff\"",
        "\"verification-bundle-locator\"",
        "\"verification-receipt-locator\"",
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
