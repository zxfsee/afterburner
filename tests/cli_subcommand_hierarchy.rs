use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

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
        usage.contains("deploy <"),
        "usage must advertise the grouped deploy subcommand"
    );
    assert!(
        !usage.contains("drift-receipt"),
        "usage must not advertise the old flat drift subcommand names"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("afterburner drift <subcommand>"),
        "README must mention the grouped drift command family"
    );
    assert!(
        readme.contains("afterburner deploy <subcommand>"),
        "README must mention the grouped deploy command family"
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
        ("drift", "receipt"),
        ("cleanup", "inventory"),
        ("profile", "environment-snapshot"),
    ];
    for (group, subcommand) in help_cases {
        let mut cmd = cargo_bin_cmd!("afterburner");
        cmd.arg(group)
            .arg(subcommand)
            .arg("--help")
            .assert()
            .success();
    }
}
