use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn eval_fails_fast_on_missing_artifact() {
    let mut cmd = cargo_bin_cmd!("eval");
    cmd.arg("does-not-exist.mpk");
    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"eval_error""#)
            .and(predicate::str::contains("does-not-exist.mpk")),
    );
}
