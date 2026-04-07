use assert_cmd::cargo::cargo_bin_cmd;

#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn main_entrypoint_stays_thin_and_deploy_dispatch_lives_in_grouped_helper() {
    let main_rs = repo_file("src/main.rs");
    assert!(
        main_rs.contains("mod cli_dispatch;"),
        "src/main.rs must declare the grouped cli_dispatch helper module"
    );
    assert!(
        main_rs.contains("cli_dispatch::run(std::env::args().skip(1))"),
        "src/main.rs must delegate argument routing to cli_dispatch::run"
    );
    for forbidden in [
        "fn run_deploy<",
        "fn run_debug_deploy<",
        "fn dispatch_deploy<",
        "\"record-verification-receipt-locator-history\"",
        "\"scheduler-heartbeat-point-rollback-supersede\"",
    ] {
        assert!(
            !main_rs.contains(forbidden),
            "src/main.rs must not retain deep deploy routing detail `{forbidden}`"
        );
    }

    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");
    for required in [
        "pub fn run_deploy",
        "pub fn run_debug_deploy",
        "\"verification-receipt\"",
        "\"record-locator-history\"",
        "\"scheduler-heartbeat\"",
        "\"pointer\"",
        "\"rollback\"",
        "\"supersession\"",
    ] {
        assert!(
            deploy_dispatch.contains(required),
            "src/cli_dispatch/deploy.rs must own deploy routing detail `{required}`"
        );
    }
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

#[test]
fn grouped_operator_grammar_stays_intent_first() {
    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("workflow-surface-check-operator-grammar:"),
        "justfile must expose `workflow-surface-check-operator-grammar:`"
    );

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "## Operator command model",
        "## Public CLI admission rule",
        "Public commands should use a resource/action shape",
        "`just workflow-surface-check-operator-grammar` keeps the grouped nested operator grammar checked.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the operator grammar rule `{needle}`"
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
            "reference index must keep grouped operator family `{needle}`"
        );
    }

    let top_help = cargo_bin_cmd!("afterburner")
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let top_usage = String::from_utf8(top_help).expect("utf8 top usage");
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
            top_usage.contains(allowed),
            "top-level usage must advertise grouped family `{allowed}`"
        );
    }

    for forbidden in [
        "|profile <...>|lineage <...>|source <...>|",
        "afterburner drift-receipt",
        "afterburner source approval-receipt",
        "afterburner source provenance-evidence-bundle",
        "afterburner record-verification-receipt-locator-history",
        "afterburner point-transport-locator",
        "afterburner record-handoff-history",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must not present flat operator grammar `{forbidden}`"
        );
        assert!(
            !reference.contains(forbidden),
            "reference index must not present flat operator grammar `{forbidden}`"
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
            !top_usage.contains(forbidden),
            "top-level usage must not advertise flat taxonomy command `{forbidden}`"
        );
        assert!(
            !reference.contains(forbidden),
            "reference index must not present flat operator grammar `{forbidden}`"
        );
    }

    let nested_help_cases = [
        ("verify", "receipt", "history"),
        ("verify", "bundle", "transport"),
        ("verify", "handoff", "reconcile"),
        ("rollback", "verification-receipt", "supersession-reconcile"),
        ("rollback", "verification-bundle", "apply"),
        ("debug", "lineage", "receipt"),
        ("debug", "lineage", "bundle"),
        ("debug", "lineage", "handoff"),
        ("debug", "lineage", "locator"),
        ("source", "approval", "receipt"),
        ("source", "provenance", "bundle"),
    ];
    for (group, family, action) in nested_help_cases {
        cargo_bin_cmd!("afterburner")
            .arg(group)
            .arg(family)
            .arg(action)
            .arg("--help")
            .assert()
            .success();
    }
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
