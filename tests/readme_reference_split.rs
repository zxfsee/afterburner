mod support;

use support::repo_file;

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
        "Text-pretraining path:",
        "Deployment stack artifact groups:",
        "Deployment verification artifact groups:",
        "Source and lineage:",
        "Distributed runtime and scheduler:",
        "Operator surface and backend fit:",
        "Profiling, provenance, and retention:",
        "Longer-horizon anchors:",
    ] {
        assert!(
            reference.contains(heading),
            "reference index must contain the grouped heading `{heading}`"
        );
    }

    for needle in [
        "`training_scalability_contract.json`",
        "`manifest.toml`",
        "`artifacts/eval/text_pretraining_eval_summary.json`",
        "`deployment_stack_check.json`",
        "`deployment_verification_receipt*.json`",
        "`artifact_cleanup_inventory.json`",
        "`profiling_provenance_receipt.json`",
        "`infer_output_drift_summary.json`",
        "`pretraining_source_approval_receipt.json`",
        "`distributed_shard_lineage_receipt.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map.",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-stack` for mechanical coverage.",
        "afterburner deploy <subcommand>",
        "artifact retention envelope",
        "remote locator contract",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the representative anchor `{needle}`"
        );
    }
}
