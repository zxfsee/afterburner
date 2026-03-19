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
fn rl_rollout_schema_fixture_has_required_contract_fields() {
    let schema_text = fs::read_to_string(fixture_path("rl_rollout_metadata.schema.json"))
        .expect("read rollout schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/rl-rollout-metadata/v1")
    );
    assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
    assert_eq!(
        schema.get("additionalProperties").and_then(|v| v.as_bool()),
        Some(false)
    );

    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("schema.required must be an array");
    let required = required
        .iter()
        .map(|v| {
            v.as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "policy_artifact",
        "policy_sha256",
        "environment",
        "created_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    assert_eq!(
        schema
            .get("properties")
            .and_then(|v| v.get("schema_version"))
            .and_then(|v| v.get("const"))
            .and_then(|v| v.as_str()),
        Some("1")
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-013"),
        "architecture decisions index must link ADR-013"
    );
    assert!(
        architecture.contains("RL"),
        "architecture must mention the RL environment fit decision"
    );

    let adr = repo_file("docs/adr/013-rl-environment-fit.md");
    assert!(
        adr.contains("single-environment"),
        "ADR-013 must describe the single-environment stance"
    );
    assert!(
        adr.contains("vectorized"),
        "ADR-013 must describe the vectorized-environment stance"
    );
    assert!(
        adr.contains("rl_rollout_metadata.schema.json"),
        "ADR-013 must anchor the decision to the rollout metadata contract"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("rl_rollout_metadata.schema.json"),
        "README must mention the RL rollout metadata contract"
    );
    assert!(
        readme.contains("vectorized"),
        "README must mention the vectorized-environment stance"
    );
}
