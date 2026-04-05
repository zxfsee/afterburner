use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

mod support;

use support::fixture_path;

#[test]
fn dashboard_snapshot_matches_fixture() {
    let expected = fs::read_to_string(fixture_path("dashboard_snapshot.txt"))
        .expect("read dashboard snapshot fixture");

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-dashboard"))
        .arg("--input")
        .arg(fixture_path("dashboard_events.jsonl"))
        .arg("--profiling-summary")
        .arg(fixture_path("dashboard_profiling_summary.fixture.json"))
        .args(["--snapshot", "--width", "80", "--height", "18"])
        .assert()
        .success();

    let stdout =
        String::from_utf8(assert.get_output().stdout.clone()).expect("dashboard stdout utf-8");
    assert_eq!(
        stdout, expected,
        "dashboard snapshot output must match the fixture-backed contract"
    );
}
