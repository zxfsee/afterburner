use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn optimized_model_capability_surface_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "optimized_model_capability_surface.schema.json",
    ))
    .expect("read optimized model capability surface schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse optimized model capability surface");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/optimized-model-capability-surface/v1")
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
        "supported_runtime_precision_contracts",
        "planning_only_optimization_steps",
        "unsupported_optimized_artifact_surfaces",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn optimized_model_capability_surface_example_and_docs_match_repo_contracts() {
    let example_text = fs::read_to_string(fixture_path(
        "optimized_model_capability_surface.example.json",
    ))
    .expect("read optimized model capability surface example");
    let example: Value =
        serde_json::from_str(&example_text).expect("parse optimized model capability surface");

    let supported = example
        .get("supported_runtime_precision_contracts")
        .and_then(Value::as_array)
        .expect("supported_runtime_precision_contracts must be an array");
    assert_eq!(
        supported.len(),
        1,
        "expected one supported runtime precision contract"
    );
    assert_eq!(
        supported[0],
        serde_json::json!({
            "weights_dtype": "f32",
            "activation_dtype": "f32",
            "quantization": "none"
        })
    );

    let planning_steps = example
        .get("planning_only_optimization_steps")
        .and_then(Value::as_array)
        .expect("planning_only_optimization_steps must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("planning step must be a string")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected_steps = ["quantization", "compression", "export", "packaging"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(planning_steps, expected_steps);

    let unsupported = example
        .get("unsupported_optimized_artifact_surfaces")
        .and_then(Value::as_array)
        .expect("unsupported_optimized_artifact_surfaces must be an array");
    assert!(
        unsupported.iter().any(|entry| {
            entry.get("surface").and_then(Value::as_str)
                == Some("reduced_precision_runtime_artifact")
                && entry.get("status").and_then(Value::as_str) == Some("unsupported")
        }),
        "example must mark reduced_precision_runtime_artifact unsupported"
    );
    assert!(
        unsupported.iter().any(|entry| {
            entry.get("surface").and_then(Value::as_str)
                == Some("compressed_optimized_artifact_package")
                && entry.get("blocked_by").and_then(Value::as_str)
                    == Some("Optimized model packaging and export contract")
        }),
        "example must tie compressed_optimized_artifact_package to the packaging contract"
    );

    let readme = repo_file("README.md");
    for needle in [
        "optimized model capability surface",
        "`weights_dtype = \"f32\"`",
        "`activation_dtype = \"f32\"`",
        "`quantization = \"none\"`",
        "planning-only",
    ] {
        assert!(
            readme.contains(needle),
            "README must mention `{needle}` for optimized model capability stance"
        );
    }

    let adr_006 = repo_file("docs/adr/006-quantized-inference-artifact-contract.md");
    for needle in [
        "optimized_model_capability_surface.schema.json",
        "`weights_dtype = \"f32\"`",
        "`activation_dtype = \"f32\"`",
        "`quantization = \"none\"`",
    ] {
        assert!(
            adr_006.contains(needle),
            "ADR-006 must mention `{needle}` as part of the current runtime capability surface"
        );
    }

    let adr_053 = repo_file("docs/adr/053-model-optimization-and-packaging-fit.md");
    for needle in [
        "optimized_model_capability_surface.schema.json",
        "planning-only",
        "compression",
        "packaging",
    ] {
        assert!(
            adr_053.contains(needle),
            "ADR-053 must mention `{needle}` as part of the optimized model capability surface"
        );
    }
}
