use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn public_cli_surface_stays_within_admitted_grouped_families() {
    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("workflow-surface-check-public-cli:"),
        "justfile must expose `workflow-surface-check-public-cli:`"
    );

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "## Operator command model",
        "## Public CLI admission rule",
        "| Deploy | `afterburner deploy <resource> ...` |",
        "| Verify | `afterburner verify <resource> ...` |",
        "| Rollback | `afterburner rollback <resource> ...` |",
        "| Inspect | `afterburner inspect <resource> ...` |",
        "| Heartbeat | `afterburner deploy scheduler-heartbeat ...` |",
        "| Debug | `afterburner debug <domain> <action> ...` |",
        "`just workflow-surface-check-public-cli` keeps the admitted public CLI family set and grouped-path guard checked.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the public CLI admission rule `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "afterburner deploy <subcommand>",
        "afterburner verify <subcommand>",
        "afterburner rollback <subcommand>",
        "afterburner drift <subcommand>",
        "afterburner cleanup <subcommand>",
        "afterburner profile <subcommand>",
        "afterburner source <subcommand>",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep grouped public family `{needle}`"
        );
    }

    let mut cmd = cargo_bin_cmd!("afterburner");
    let output = cmd
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let usage = String::from_utf8(output).expect("utf8 usage");
    for allowed in [
        "deploy <",
        "verify <",
        "rollback <",
        "drift <",
        "cleanup <",
        "profile <",
        "source <",
        "debug <",
    ] {
        assert!(
            usage.contains(allowed),
            "top-level usage must advertise grouped family `{allowed}`"
        );
    }

    for forbidden in [
        "drift-receipt",
        "approval-receipt",
        "provenance-evidence-bundle",
        "record-verification-receipt-locator-history",
        "point-transport-locator",
        "record-handoff-history",
    ] {
        assert!(
            !usage.contains(forbidden),
            "top-level usage must not advertise flat taxonomy command `{forbidden}`"
        );
        assert!(
            !reference.contains(forbidden),
            "reference index must not present flat taxonomy command `{forbidden}` as public surface"
        );
    }
}
