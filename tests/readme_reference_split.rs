use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn readme_points_to_reference_docs_and_stays_navigation_first() {
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
            "reference index must contain `{heading}`"
        );
    }
    for needle in [
        "artifact_upload_request.json",
        "Deployment verification artifact groups:",
        "Rollout verification and pointer state:",
        "Pretraining source artifact groups:",
        "Distributed shard lineage artifact groups:",
        "`deployment_verification_evidence_bundle*.json`",
        "infer_output_drift_baseline_pointer.json",
        "profiling_provenance_evidence_bundle.json",
        "distributed_shard_lineage_evidence_handoff.json",
        "remote locator contract",
        "deployment verification evidence provenance",
        "artifact retention envelope",
        "afterburner deploy <subcommand>",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must contain `{needle}`"
        );
    }
}
