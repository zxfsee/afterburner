use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn infer_fails_fast_on_missing_artifact() {
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer").arg("does-not-exist.mpk");
    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"infer_error""#)
            .and(predicate::str::contains(r#""kind":"artifact_missing""#))
            .and(predicate::str::contains(r#""artifact":"#))
            .and(predicate::str::contains("does-not-exist.mpk")),
    );
}
