use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;

#[test]
fn eval_fails_fast_on_missing_artifact() {
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("eval").arg("does-not-exist.mpk");
    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"eval_error""#)
            .and(predicate::str::contains("does-not-exist.mpk")),
    );
}

#[test]
fn eval_accepts_inline_artifact_flag() {
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("eval").arg("--artifact=does-not-exist-inline.mpk");
    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"eval_error""#)
            .and(predicate::str::contains("does-not-exist-inline.mpk"))
            .and(predicate::str::contains("unknown argument").not()),
    );
}

#[test]
fn eval_default_artifact_path_uses_current_version_pointer() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let inference_dir = tmp.path().join("artifacts").join("inference");
    fs::create_dir_all(&inference_dir).expect("create inference dir");
    fs::write(inference_dir.join("current"), "9.9.9\n").expect("write current pointer");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.current_dir(tmp.path());
    cmd.arg("eval");
    cmd.assert()
        .failure()
        .code(2)
        .stderr(
            predicate::str::contains(r#""event":"eval_error""#).and(predicate::str::contains(
                "artifacts/inference/9.9.9/model.mpk",
            )),
        );
}
