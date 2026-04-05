use std::fs;
use std::path::PathBuf;

use afterburner::manifest::ArtifactManifest;
use serde_json::{Value, json};

mod support;

use support::fixture_path;

fn artifact_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn profiling_summary_cli_matches_fixture() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");
    let weights_path = artifact_dir.join("model.mpk");
    fs::write(&weights_path, b"fixture-model").expect("write weights placeholder");
    ArtifactManifest::for_current("model.mpk", "0.1.0", "fixture-checksum".to_string())
        .write_to_dir(&artifact_dir)
        .expect("write fixture manifest");

    let output = tmp.path().join("profiling_summary.json");
    let expected = fs::read_to_string(fixture_path("profiling_hotspot_summary.fixture.json"))
        .expect("read profiling summary fixture");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("afterburner_profile_summary");
    cmd
        .arg("--input")
        .arg(fixture_path("profiling_flamegraph_fixture.svg"))
        .arg("--output")
        .arg(&output)
        .arg("--weights-artifact")
        .arg(&weights_path)
        .arg("--backend")
        .arg("wgpu")
        .arg("--profile-command")
        .arg("cargo flamegraph --dev --deterministic --bin afterburner -- infer artifacts/inference/0.1.0/model.mpk")
        .assert()
        .success();

    let actual = fs::read_to_string(&output).expect("read generated profiling summary");
    let mut actual: Value =
        serde_json::from_str(&actual).expect("parse generated profiling summary");
    let expected: Value = serde_json::from_str(&expected).expect("parse profiling summary fixture");

    let object = actual
        .as_object_mut()
        .expect("profiling summary must be a top-level object");
    object.insert(
        "weights_artifact".to_string(),
        json!("artifacts/inference/0.1.0/model.mpk"),
    );
    object.insert("flamegraph_svg".to_string(), json!("/tmp/fixture.svg"));

    assert_eq!(
        actual, expected,
        "profiling summary output must match the fixture-backed contract"
    );
}

#[test]
fn profiling_summary_schema_and_sample_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("profiling_hotspot_summary.schema.json"))
        .expect("read profiling hotspot summary schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse profiling schema");
    let schema = schema
        .as_object()
        .expect("profiling hotspot schema must be a top-level object");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/profiling-hotspot-summary/v1")
    );
    assert_eq!(schema.get("type").and_then(Value::as_str), Some("object"));

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array");
    for key in [
        "schema_version",
        "profile_kind",
        "profiler",
        "weights_artifact",
        "artifact_version",
        "backend",
        "profile_command",
        "flamegraph_svg",
        "total_samples",
        "top_hotspots",
    ] {
        assert!(
            required.iter().any(|value| value.as_str() == Some(key)),
            "schema.required must include `{key}`"
        );
    }

    let sample_text = fs::read_to_string(artifact_path("infer_hotspot_summary.fixture.json"))
        .expect("read profiling summary artifact");
    let sample: Value =
        serde_json::from_str(&sample_text).expect("parse profiling summary artifact");
    let sample = sample
        .as_object()
        .expect("profiling summary artifact must be a top-level object");

    assert_eq!(
        sample.get("schema_version").and_then(Value::as_str),
        Some("1")
    );
    assert_eq!(
        sample.get("profile_kind").and_then(Value::as_str),
        Some("infer")
    );
    assert_eq!(
        sample.get("profiler").and_then(Value::as_str),
        Some("cargo-flamegraph")
    );
    assert_eq!(sample.get("backend").and_then(Value::as_str), Some("wgpu"));

    let hotspots = sample
        .get("top_hotspots")
        .and_then(Value::as_array)
        .expect("top_hotspots must be an array");
    assert!(
        !hotspots.is_empty(),
        "profiling summary artifact must include at least one hotspot"
    );
    assert!(
        hotspots.iter().any(|value| {
            value
                .get("symbol")
                .and_then(Value::as_str)
                .is_some_and(|symbol| symbol.contains("afterburner::cmd_infer::run_infer"))
        }),
        "profiling summary artifact must include the infer execution hotspot"
    );
}
