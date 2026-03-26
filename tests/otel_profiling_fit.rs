use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
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
    assert!(
        architecture.contains("OpenTelemetry"),
        "architecture must mention the profiling/OpenTelemetry fit decision"
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
