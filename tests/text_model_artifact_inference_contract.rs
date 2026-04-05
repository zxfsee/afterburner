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
fn text_inference_profile_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path("text_inference_profile.schema.json"))
        .expect("read text inference profile schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/text-inference-profile/v1")
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
        "task",
        "tokenizer_profile",
        "max_context_tokens",
        "sampling_defaults",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn text_model_artifact_and_inference_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-052"),
        "architecture decisions index must link ADR-052"
    );

    let adr = repo_file("docs/adr/052-text-model-artifact-and-inference-contract.md");
    for needle in [
        "text_inference_profile.schema.json",
        "causal-lm",
        "`afterburner sample --artifact PATH --prompt TEXT --max-new-tokens N`",
        "`text_sample_done`",
        "`prompt_token_count`",
        "`generated_token_count`",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-052 must mention `{needle}` as part of the text inference contract"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains(
            "Text-trained models should not reuse the current MNIST/logits infer surface"
        ),
        "reference index must mention the separate text inference surface"
    );
    assert!(
        readme.contains("text inference profile sidecar"),
        "reference index must mention the text inference sidecar"
    );
}
