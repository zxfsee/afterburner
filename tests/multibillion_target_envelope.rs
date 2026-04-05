mod support;

use support::repo_file;

#[test]
fn multibillion_target_envelope_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-015"),
        "architecture decisions index must link ADR-015"
    );
    let adr = repo_file("docs/adr/015-multibillion-target-envelope.md");
    assert!(
        adr.contains("distributed_shard_metadata.schema.json"),
        "ADR-015 must reference the shard metadata contract"
    );
    assert!(
        adr.contains("training_scalability_contract.json"),
        "ADR-015 must reference the training scalability contract"
    );
    assert!(
        adr.contains("deployment_target_profile.schema.json"),
        "ADR-015 must reference the deployment target profile contract"
    );
    assert!(
        adr.contains("artifact_upload_request.schema.json"),
        "ADR-015 must reference the upload request contract"
    );
    assert!(
        adr.contains("precision"),
        "ADR-015 must describe the precision constraint"
    );

    let readme = repo_file("docs/reference.md");
    for needle in ["multibillion", "distributed_shard_metadata.schema.json"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the target-envelope anchor `{needle}`"
        );
    }
}
