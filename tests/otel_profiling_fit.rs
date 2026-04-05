use std::fs;

use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

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
