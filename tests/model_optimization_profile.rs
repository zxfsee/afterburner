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
fn model_optimization_profile_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path("model_optimization_profile.schema.json"))
        .expect("read optimization profile schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/model-optimization-profile/v1")
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
    let expected = [
        "schema_version",
        "input_artifact_version",
        "target_environment",
        "optimization_steps",
        "output_constraints",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn model_optimization_and_packaging_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-053"),
        "architecture decisions index must link ADR-053"
    );

    let adr = repo_file("docs/adr/053-model-optimization-and-packaging-fit.md");
    for needle in [
        "model_optimization_profile.schema.json",
        "quantization",
        "compression",
        "export",
        "packaging",
        "output_constraints",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-053 must mention `{needle}` as part of the optimization fit decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("separate post-training pipeline"),
        "reference index must mention the optimization/package pipeline stance"
    );
}
