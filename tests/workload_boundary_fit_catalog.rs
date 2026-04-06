use std::collections::BTreeSet;
use std::fs;

use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn fineweb_edu_source_adoption_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-049"),
        "architecture decisions index must link ADR-049"
    );
    let adr = repo_file("docs/adr/049-fineweb-edu-source-adoption.md");
    for needle in [
        "`fineweb-edu/slice`",
        "`source_revision`",
        "bounded local slice snapshot",
        "tens of gigabytes",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-049 must mention `{needle}` as part of the source fit decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("fineweb-edu/slice"),
        "reference index must mention the FineWeb-Edu source fit"
    );
}

#[test]
fn gateway_api_inference_extension_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-065"),
        "architecture decisions index must link ADR-065"
    );
    for needle in [
        "Gateway API inference extension",
        "traffic-management",
        "thin adapter target",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the Gateway API fit decision"
        );
    }

    let adr = repo_file("docs/adr/065-gateway-api-inference-extension-fit.md");
    for needle in [
        "Do not adopt the Gateway API inference extension now.",
        "traffic-management reference",
        "thin adapter target",
        "adapter-side",
        "runtime, scheduler, or deployment stack contracts",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-065 must mention `{needle}` as part of the Gateway API fit decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("Gateway API inference extension"),
        "README must mention the current Gateway API inference extension fit stance"
    );
}

#[test]
fn macbook_text_pretraining_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-047"),
        "architecture decisions index must link ADR-047"
    );
    let adr = repo_file("docs/adr/047-macbook-text-pretraining-strategy.md");
    for needle in [
        "decoder-only language model",
        "`50M` to `300M`",
        "`1024`",
        "`50M` to `200M`",
        "`AdamW`",
        "checkpoint cadence",
        "tiny-overfit",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-047 must mention `{needle}` as part of the text-pretraining strategy"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "decoder-only language model",
        "single-node, single-device execution",
        "`50M` to `300M` parameters",
        "`1024` token context length",
        "`50M` to `200M` token budgets",
        "`AdamW`",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must mention the bounded MacBook text-pretraining detail `{needle}`"
        );
    }
}

#[test]
fn multibillion_target_envelope_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-015"),
        "architecture decisions index must link ADR-015"
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

    let readme = repo_file("docs/reference.md");
    for needle in ["multibillion", "distributed_shard_metadata.schema.json"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the target-envelope anchor `{needle}`"
        );
    }
}

#[test]
fn otel_profiling_correlation_fit_is_documented() {
    let schema_text = fs::read_to_string(fixture_path("profiling_hotspot_summary.schema.json"))
        .expect("read profiling hotspot summary schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse profiling schema");
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array");

    for key in ["artifact_version", "backend", "profile_command"] {
        assert!(
            required.iter().any(|value| value.as_str() == Some(key)),
            "profiling schema must include `{key}`"
        );
    }

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-016"),
        "architecture decisions index must link ADR-016"
    );
    let adr = repo_file("docs/adr/016-otel-profiling-fit.md");
    assert!(
        adr.contains("profiling_hotspot_summary.schema.json"),
        "ADR-016 must reference the profiling summary contract"
    );
    assert!(
        adr.contains("artifact_version"),
        "ADR-016 must mention existing profiling identity fields"
    );
    assert!(
        adr.contains("SDK"),
        "ADR-016 must state the no-heavy-SDK stance"
    );

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("OpenTelemetry"),
        "reference index must mention the OpenTelemetry profiling fit stance"
    );
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
