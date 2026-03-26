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
fn text_tokenizer_packing_profile_schema_is_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("text_tokenizer_packing_profile.schema.json"))
            .expect("read tokenizer profile schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/text-tokenizer-packing-profile/v1")
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
        "tokenizer_id",
        "tokenizer_revision",
        "vocab_size",
        "bos_token",
        "eos_token",
        "pad_token",
        "context_length",
        "truncation_policy",
        "packing_policy",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let example_text =
        fs::read_to_string(fixture_path("text_tokenizer_packing_profile.example.json"))
            .expect("read tokenizer profile example");
    let example: serde_json::Value =
        serde_json::from_str(&example_text).expect("parse example json");
    let example = example.as_object().expect("example must be object");
    for key in &expected {
        assert!(
            example.contains_key(key),
            "example tokenizer profile must include `{key}`"
        );
    }
}

#[test]
fn text_tokenizer_packing_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-050"),
        "architecture decisions index must link ADR-050"
    );
    assert!(
        architecture.contains("tokenizer and packing surface"),
        "architecture must mention the tokenizer/packing surface"
    );

    let adr = repo_file("docs/adr/050-text-tokenizer-and-packing-contract.md");
    for needle in [
        "text_tokenizer_packing_profile.schema.json",
        "tokenizer_id",
        "tokenizer_revision",
        "bos_token",
        "eos_token",
        "pad_token",
        "context_length",
        "truncation_policy",
        "packing_policy",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-050 must mention `{needle}` as part of the tokenizer contract"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("tokenizer and packing side"),
        "reference index must mention the tokenizer/packing profile stance"
    );
}
