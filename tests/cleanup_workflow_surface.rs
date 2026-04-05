#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn cleanup_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "cleanup-inventory:",
        "cleanup-policy profile:",
        "cleanup-dry-run inventory policy generated_at_unix_ms:",
        "cleanup-execute dry_run_receipt artifacts_root executed_at_unix_ms:",
        "cleanup-evidence-bundle execution_receipt:",
        "workflow-surface-check-cleanup:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Cleanup stays grouped under these operator flows:",
        "Scope and policy: `just cleanup-inventory`, `just cleanup-policy`.",
        "Plan and execute: `just cleanup-dry-run`, `just cleanup-execute`.",
        "Evidence pack: `just cleanup-evidence-bundle`.",
        "`just workflow-surface-check-cleanup` guards the grouped cleanup workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped cleanup surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just cleanup-inventory` writes `artifact_cleanup_inventory.json`.",
        "- `just cleanup-policy` writes `artifact_cleanup_policy.json`.",
        "- `just cleanup-dry-run` writes `artifact_cleanup_dry_run_receipt.json`.",
        "- `just cleanup-execute` writes `artifact_cleanup_execution_receipt.json`.",
        "- `just cleanup-evidence-bundle` writes `artifact_cleanup_evidence_bundle.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating cleanup variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Cleanup:",
        "`artifact_cleanup_inventory.json`",
        "`artifact_cleanup_policy.json`",
        "`artifact_cleanup_dry_run_receipt.json`",
        "`artifact_cleanup_execution_receipt.json`",
        "`artifact_cleanup_evidence_bundle.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-cleanup` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped cleanup surface `{needle}`"
        );
    }
}
