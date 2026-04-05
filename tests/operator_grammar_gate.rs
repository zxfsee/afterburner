use assert_cmd::cargo::cargo_bin_cmd;

#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

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
