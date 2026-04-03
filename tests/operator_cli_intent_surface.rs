use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn deployment_verification_operator_surface_uses_intent_families() {
    cargo_bin_cmd!("afterburner")
        .arg("verify")
        .arg("receipt")
        .arg("--help")
        .assert()
        .success();

    cargo_bin_cmd!("afterburner")
        .arg("rollback")
        .arg("verification-bundle")
        .arg("--help")
        .assert()
        .success();

    cargo_bin_cmd!("afterburner")
        .arg("deploy")
        .arg("verification-receipt")
        .arg("--help")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "use `afterburner verify receipt ...`",
        ));

    cargo_bin_cmd!("afterburner")
        .arg("deploy")
        .arg("record-verification-bundle-rollback-history")
        .arg("--help")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "use `afterburner debug deploy verification-bundle record-rollback-history ...`",
        ));

    cargo_bin_cmd!("afterburner")
        .arg("debug")
        .arg("deploy")
        .arg("record-verification-bundle-rollback-history")
        .arg("--help")
        .assert()
        .failure();

    cargo_bin_cmd!("afterburner")
        .arg("debug")
        .arg("deploy")
        .arg("verification-bundle")
        .arg("record-rollback-history")
        .arg("--help")
        .assert()
        .success();
}
