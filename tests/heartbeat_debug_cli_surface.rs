use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn heartbeat_low_level_commands_move_under_debug_deploy() {
    let mut public_cmd = cargo_bin_cmd!("afterburner");
    public_cmd
        .arg("deploy")
        .arg("scheduler-heartbeat-supersede")
        .arg("--help")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "afterburner debug deploy scheduler-heartbeat-supersede",
        ));

    let mut debug_cmd = cargo_bin_cmd!("afterburner");
    debug_cmd
        .arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat-supersede")
        .arg("--help")
        .assert()
        .success();
}
