#[path = "support/markdown.rs"]
mod markdown_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use markdown_test_support::markdown_section;
use repo_test_support::repo_file;

#[test]
fn readme_points_to_reference_docs_and_keeps_reference_grouped() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("[Reference index](./docs/reference.md)"),
        "README frontpage must point readers to the reference index"
    );
    assert!(
        readme.contains(
            "Deeper contract and capability detail now lives in [docs/reference.md](./docs/reference.md)."
        ),
        "README must keep the deeper reference split explicit"
    );

    let reference = repo_file("docs/reference.md");
    for heading in [
        "## Training and inference contracts",
        "## Workflow and artifact families",
        "## Capability and stance summary",
    ] {
        assert!(
            reference.contains(heading),
            "reference index must contain the grouped heading `{heading}`"
        );
    }

    let training = markdown_section(&reference, "## Training and inference contracts");
    for needle in [
        "Text-pretraining path:",
        "`training_scalability_contract.json`",
        "`manifest.toml`",
        "`artifacts/eval/text_pretraining_eval_summary.json`",
    ] {
        assert!(
            training.contains(needle),
            "training/inference section must keep the representative anchor `{needle}`"
        );
    }

    let workflows = markdown_section(&reference, "## Workflow and artifact families");
    for needle in [
        "Deployment stack artifact groups:",
        "`deployment_stack_check.json`",
        "Deployment verification artifact groups:",
        "`deployment_verification_receipt*.json`",
        "Source and lineage:",
        "`pretraining_source_approval_receipt.json`",
        "`distributed_shard_lineage_receipt.json`",
        "`artifact_cleanup_inventory.json`",
        "`profiling_provenance_receipt.json`",
        "`infer_output_drift_summary.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map.",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-stack` for mechanical coverage.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow/artifact families section must keep the representative anchor `{needle}`"
        );
    }

    let capability_summary = markdown_section(&reference, "## Capability and stance summary");
    for needle in [
        "Distributed runtime and scheduler:",
        "afterburner deploy <subcommand>",
        "Operator surface and backend fit:",
        "Profiling, provenance, and retention:",
        "artifact retention envelope",
        "Longer-horizon anchors:",
        "remote locator contract",
    ] {
        assert!(
            capability_summary.contains(needle),
            "capability summary must keep the representative anchor `{needle}`"
        );
    }
}
