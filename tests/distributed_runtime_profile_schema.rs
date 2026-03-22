use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn distributed_runtime_profile_schema_fixture_has_required_contract_fields() {
    let schema_text = fs::read_to_string(fixture_path("distributed_runtime_profile.schema.json"))
        .expect("read distributed runtime profile schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/distributed-runtime-profile/v1")
    );
    assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
    assert_eq!(
        schema.get("additionalProperties").and_then(|v| v.as_bool()),
        Some(false)
    );

    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("schema.required must be an array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "artifact_version",
        "model_size",
        "node_count",
        "gpus_per_node",
        "world_size",
        "parallelism",
        "metrics",
        "runtime_settings",
        "environment_fingerprint",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let example_text = fs::read_to_string(fixture_path("distributed_runtime_profile.example.json"))
        .expect("read distributed runtime profile example");
    let example: serde_json::Value =
        serde_json::from_str(&example_text).expect("parse example json");
    let example = example.as_object().expect("example must be an object");
    for key in &expected {
        assert!(
            example.contains_key(key),
            "example distributed runtime profile must include `{key}`"
        );
    }
}

#[test]
fn distributed_runtime_profile_schema_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-043"),
        "architecture decisions index must link ADR-043"
    );
    assert!(
        architecture.contains("distributed runtime profile trials"),
        "architecture must mention the distributed runtime profile artifact"
    );

    let adr = repo_file("docs/adr/043-distributed-runtime-profile-schema.md");
    for needle in [
        "distributed_runtime_profile.schema.json",
        "model_size",
        "node_count",
        "gpus_per_node",
        "world_size",
        "dp",
        "tp",
        "pp",
        "sp_cp",
        "ep",
        "gas",
        "microbatch_size",
        "zero_stage",
        "mfu",
        "max_memory_bytes",
        "environment fingerprint",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-043 must mention `{needle}` as part of the profile schema decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("distributed runtime trials"),
        "README must mention the distributed runtime profile artifact stance"
    );
}
