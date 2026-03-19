use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn multibillion_target_envelope_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-015"),
        "architecture decisions index must link ADR-015"
    );
    assert!(
        architecture.contains("multibillion"),
        "architecture must mention the multibillion-scale target envelope"
    );
    assert!(
        architecture.contains("distributed_shard_metadata"),
        "architecture must anchor the target envelope to distributed shard metadata"
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

    let readme = repo_file("README.md");
    assert!(
        readme.contains("multibillion"),
        "README must mention the multibillion-scale target envelope"
    );
    assert!(
        readme.contains("distributed_shard_metadata.schema.json"),
        "README must mention the shard metadata contract"
    );
}
