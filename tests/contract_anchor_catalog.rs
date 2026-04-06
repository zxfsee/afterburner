#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn artifact_retention_envelope_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-019"),
        "architecture decisions index must link ADR-019"
    );
    let adr = repo_file("docs/adr/019-artifact-retention-envelope.md");
    for needle in [
        "artifacts/inference/current",
        "artifacts/inference/<version>/",
        "artifacts/deploy/",
        "artifacts/train/",
        "artifacts/profiling/",
        "prune",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-019 must mention `{needle}` in the retention envelope"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("retention envelope"),
        "reference index must mention the artifact retention envelope"
    );
    for needle in ["artifacts/inference/current", "artifacts/profiling/"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the retention anchor `{needle}`"
        );
    }
}

#[test]
fn distributed_checkpoint_index_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-018"),
        "architecture decisions index must link ADR-018"
    );

    let adr = repo_file("docs/adr/018-distributed-checkpoint-index.md");
    assert!(
        adr.contains("distributed_shard_metadata.schema.json"),
        "ADR-018 must reference the shard metadata contract"
    );
    assert!(
        adr.contains("artifact_version"),
        "ADR-018 must mention artifact_version as part of the checkpoint index"
    );
    assert!(
        adr.contains("checkpoint_root"),
        "ADR-018 must mention checkpoint_root as part of the checkpoint index"
    );
    assert!(
        adr.contains("index-only"),
        "ADR-018 must make the index-only stance explicit"
    );

    let reference = repo_file("docs/reference.md");
    for needle in ["checkpoint index", "distributed_shard_metadata"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the distributed checkpoint index anchor `{needle}`"
        );
    }
}

#[test]
fn distributed_optimizer_and_checkpoint_state_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-061"),
        "architecture decisions index must link ADR-061"
    );
    let adr = repo_file("docs/adr/061-distributed-optimizer-and-checkpoint-state.md");
    for needle in [
        "optimizer-state",
        "checkpoint_group",
        "scheduler",
        "stop/resume",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-061 must mention `{needle}` as part of the recovery decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("checkpoint_group"),
        "README must mention the explicit checkpoint_group recovery anchor"
    );
    let reference = repo_file("docs/reference.md");
    for needle in ["distributed optimizer-state recovery", "checkpoint_group"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the optimizer/checkpoint anchor `{needle}`"
        );
    }
}

#[test]
fn distributed_shard_lineage_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-023"),
        "architecture decisions index must link ADR-023"
    );

    let adr = repo_file("docs/adr/023-distributed-shard-lineage.md");
    for needle in [
        "distributed_shard_metadata.schema.json",
        "source",
        "source_revision",
        "checkpoint_group",
        "shard_id",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-023 must mention `{needle}` as part of shard lineage"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Distributed shard lineage artifact groups",
        "source_revision",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must mention the distributed shard lineage anchor `{needle}`"
        );
    }
}

#[test]
fn pretraining_source_registry_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-021"),
        "architecture decisions index must link ADR-021"
    );
    let adr = repo_file("docs/adr/021-pretraining-source-registry.md");
    assert!(
        adr.contains("pretraining_dataset_manifest.schema.json"),
        "ADR-021 must reference the dataset manifest contract"
    );
    for needle in [
        "`source`",
        "`source_revision`",
        "`upstream_locator`",
        "`license`",
        "`approval_status`",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-021 must mention {needle} as part of the source registry decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in ["source registry", "source_revision"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the source registry anchor `{needle}`"
        );
    }
}

#[test]
fn profiling_environment_provenance_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-025"),
        "architecture decisions index must link ADR-025"
    );
    let adr = repo_file("docs/adr/025-profiling-environment-provenance.md");
    for needle in [
        "profiling_hotspot_summary.schema.json",
        "host_os",
        "host_arch",
        "profiler_version",
        "profiler_path",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-025 must mention `{needle}` as part of profiling provenance"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "profiling environment provenance",
        "profiling_provenance_receipt.json",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must mention the profiling environment provenance anchor `{needle}`"
        );
    }
}

#[test]
fn deployment_verification_evidence_provenance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-027"),
        "architecture decisions index must link ADR-027"
    );
    let adr = repo_file("docs/adr/027-deployment-verification-evidence-provenance.md");
    for needle in [
        "deployment_verification_receipt.json",
        "evidence_sources",
        "artifact_path",
        "event_name",
        "observed_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-027 must mention `{needle}` as part of evidence provenance"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "deployment verification evidence provenance",
        "deployment verification receipt",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must mention the deployment verification provenance anchor `{needle}`"
        );
    }
}
