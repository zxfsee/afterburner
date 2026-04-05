use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn main_entrypoint_stays_thin_and_deploy_dispatch_lives_in_grouped_helper() {
    let main_rs = repo_file("src/main.rs");
    assert!(
        main_rs.contains("mod cli_dispatch;"),
        "src/main.rs must declare the grouped cli_dispatch helper module"
    );
    assert!(
        main_rs.contains("cli_dispatch::run(std::env::args().skip(1))"),
        "src/main.rs must delegate argument routing to cli_dispatch::run"
    );
    for forbidden in [
        "fn run_deploy<",
        "fn run_debug_deploy<",
        "fn dispatch_deploy<",
        "\"record-verification-receipt-locator-history\"",
        "\"scheduler-heartbeat-point-rollback-supersede\"",
    ] {
        assert!(
            !main_rs.contains(forbidden),
            "src/main.rs must not retain deep deploy routing detail `{forbidden}`"
        );
    }

    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");
    for required in [
        "pub fn run_deploy",
        "pub fn run_debug_deploy",
        "\"verification-receipt\"",
        "\"record-locator-history\"",
        "\"scheduler-heartbeat\"",
        "\"pointer\"",
        "\"rollback\"",
        "\"supersession\"",
    ] {
        assert!(
            deploy_dispatch.contains(required),
            "src/cli_dispatch/deploy.rs must own deploy routing detail `{required}`"
        );
    }
}
