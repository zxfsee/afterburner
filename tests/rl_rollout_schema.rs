use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

mod support;

use support::{fixture_path, repo_file};

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

    let readme = repo_file("docs/reference.md");
    for needle in ["rl_rollout_metadata.schema.json", "vectorized", "RL"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the RL fit detail `{needle}`"
        );
    }
}
