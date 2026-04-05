use assert_cmd::cargo::cargo_bin_cmd;

mod support;

use support::repo_file;

#[test]
fn cli_usage_and_docs_prefer_grouped_subcommands() {
    let mut cmd = cargo_bin_cmd!("afterburner");
    let output = cmd
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let usage = String::from_utf8(output).expect("utf8 usage");
    assert!(
        usage.contains("drift <"),
        "usage must advertise the grouped drift subcommand"
    );
    assert!(
        usage.contains("cleanup <"),
        "usage must advertise the grouped cleanup subcommand"
    );
    assert!(
        usage.contains("profile <"),
        "usage must advertise the grouped profile subcommand"
    );
    assert!(
        usage.contains("source <"),
        "usage must advertise the grouped source subcommand"
    );
    assert!(
        usage.contains("deploy <"),
        "usage must advertise the grouped deploy subcommand"
    );
    assert!(
        usage.contains("verify <"),
        "usage must advertise the grouped verify subcommand"
    );
    assert!(
        usage.contains("rollback <"),
        "usage must advertise the grouped rollback subcommand"
    );
    assert!(
        !usage.contains("drift-receipt"),
        "usage must not advertise the old flat drift subcommand names"
    );

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("afterburner drift <subcommand>"),
        "reference index must mention the grouped drift command family"
    );
    assert!(
        readme.contains("afterburner deploy <subcommand>"),
        "reference index must mention the grouped deploy command family"
    );
    assert!(
        readme.contains("afterburner verify <subcommand>"),
        "reference index must mention the grouped verify command family"
    );
    assert!(
        readme.contains("afterburner rollback <subcommand>"),
        "reference index must mention the grouped rollback command family"
    );
    assert!(
        readme.contains("afterburner source <subcommand>"),
        "reference index must mention the grouped source command family"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-044"),
        "architecture decisions index must link ADR-044"
    );
}

#[test]
fn grouped_subcommands_dispatch_to_existing_tools() {
    let help_cases = [
        ("deploy", "upload"),
        ("verify", "receipt"),
        ("rollback", "verification-bundle"),
        ("drift", "receipt"),
        ("cleanup", "inventory"),
        ("profile", "environment-snapshot"),
        ("source", "approval"),
    ];
    for (group, subcommand) in help_cases {
        let mut cmd = cargo_bin_cmd!("afterburner");
        cmd.arg(group)
            .arg(subcommand)
            .arg("--help")
            .assert()
            .success();
    }

    let nested_help_cases = [
        ("verify", "receipt", "history"),
        ("verify", "bundle", "transport"),
        ("verify", "handoff", "reconcile"),
        ("rollback", "verification-receipt", "supersession-reconcile"),
        ("rollback", "verification-bundle", "apply"),
        ("source", "approval", "receipt"),
        ("source", "provenance", "bundle"),
    ];
    for (group, family, action) in nested_help_cases {
        let mut cmd = cargo_bin_cmd!("afterburner");
        cmd.arg(group)
            .arg(family)
            .arg(action)
            .arg("--help")
            .assert()
            .success();
    }
}
