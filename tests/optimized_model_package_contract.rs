use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn optimized_model_package_contract_schema_is_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("optimized_model_package_contract.schema.json"))
            .expect("read optimized model package contract schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse optimized model package contract");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/optimized-model-package-contract/v1")
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "input_artifact_version",
        "target_environment",
        "optimization_steps",
        "runtime_precision_contract",
        "optimized_artifact_path",
        "optimized_artifact_sha256",
        "export_format",
        "package_root",
        "package_layout",
        "packaging_inputs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn optimized_model_package_contract_example_and_docs_are_explicit() {
    let example_text = fs::read_to_string(fixture_path(
        "optimized_model_package_contract.example.json",
    ))
    .expect("read optimized model package contract example");
    let example: Value =
        serde_json::from_str(&example_text).expect("parse optimized model package contract");

    assert_eq!(
        example.get("runtime_precision_contract"),
        Some(&serde_json::json!({
            "weights_dtype": "f32",
            "activation_dtype": "f32",
            "quantization": "none"
        }))
    );
    assert_eq!(
        example.get("export_format").and_then(Value::as_str),
        Some("burn-mpk")
    );

    let layout_roles = example
        .get("package_layout")
        .and_then(Value::as_array)
        .expect("package_layout must be an array")
        .iter()
        .map(|entry| {
            entry
                .get("role")
                .and_then(Value::as_str)
                .expect("package_layout.role must be a string")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected_roles = [
        "manifest",
        "weights",
        "optimization_profile",
        "package_contract",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(layout_roles, expected_roles);

    let packaging_inputs = example
        .get("packaging_inputs")
        .and_then(Value::as_object)
        .expect("packaging_inputs must be an object");
    for key in [
        "base_manifest_path",
        "base_weights_path",
        "optimization_profile_path",
    ] {
        assert!(
            packaging_inputs.get(key).and_then(Value::as_str).is_some(),
            "packaging_inputs must include `{key}`"
        );
    }

    let readme = repo_file("README.md");
    for needle in [
        "optimized model package contract",
        "`optimized_model_package_contract.schema.json`",
        "`export_format`",
        "`packaging_inputs`",
    ] {
        assert!(
            readme.contains(needle),
            "README must mention `{needle}` for the packaging contract"
        );
    }

    let adr = repo_file("docs/adr/053-model-optimization-and-packaging-fit.md");
    for needle in [
        "optimized_model_package_contract.schema.json",
        "export_format",
        "packaging_inputs",
        "package_layout",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-053 must mention `{needle}` as part of the optimized packaging contract"
        );
    }
}
