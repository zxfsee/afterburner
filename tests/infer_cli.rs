use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn infer_fails_fast_on_missing_artifact() {
    let missing = "does-not-exist.mpk";

    let mut cmd = cargo_bin_cmd!("infer");
    cmd.arg(missing);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("inference artifact not found"))
        .stderr(predicate::str::contains(missing));
}
