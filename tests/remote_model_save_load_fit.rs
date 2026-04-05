use std::collections::BTreeSet;
use std::fs;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn remote_model_save_load_fit_is_documented() {
    let schema_text = fs::read_to_string(fixture_path("remote_artifact_locator.schema.json"))
        .expect("read remote locator schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/remote-artifact-locator/v1")
    );
    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("schema.required must be array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = ["schema_version", "provider", "locator", "mode"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-051"),
        "architecture decisions index must link ADR-051"
    );
    assert!(
        architecture.contains("provider-neutral"),
        "architecture must mention the provider-neutral remote locator stance"
    );

    let adr = repo_file("docs/adr/051-remote-model-save-load-fit.md");
    for needle in ["provider", "locator", "mode", "storage SDK"] {
        assert!(
            adr.contains(needle),
            "ADR-051 must mention `{needle}` as part of the remote fit decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("remote locator contract"),
        "reference index must mention the remote save/load locator stance"
    );
}
