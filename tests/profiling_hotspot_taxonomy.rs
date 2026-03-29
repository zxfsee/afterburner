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
fn profiling_hotspot_taxonomy_is_documented() {
    let fixture_text = fs::read_to_string(fixture_path("profiling_hotspot_summary.fixture.json"))
        .expect("read profiling hotspot summary fixture");
    let fixture: Value =
        serde_json::from_str(&fixture_text).expect("parse profiling hotspot summary fixture");
    let hotspots = fixture
        .get("top_hotspots")
        .and_then(Value::as_array)
        .expect("top_hotspots must be an array");

    assert!(
        hotspots.iter().any(|value| {
            value
                .get("symbol")
                .and_then(Value::as_str)
                .is_some_and(|symbol| symbol.contains("afterburner::cmd_infer::run_infer"))
        }),
        "fixture must include an execution hotspot"
    );
    assert!(
        hotspots.iter().any(|value| {
            value
                .get("symbol")
                .and_then(Value::as_str)
                .is_some_and(|symbol| symbol.contains("burn_tensor"))
        }),
        "fixture must include a framework hotspot"
    );
    assert!(
        hotspots.iter().any(|value| {
            value
                .get("symbol")
                .and_then(Value::as_str)
                .is_some_and(|symbol| symbol.contains("cubecl"))
        }),
        "fixture must include a compiler/runtime hotspot"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-020"),
        "architecture decisions index must link ADR-020"
    );
    assert!(
        architecture.contains("hotspot taxonomy"),
        "architecture must mention the profiling hotspot taxonomy"
    );

    let adr = repo_file("docs/adr/020-profiling-hotspot-taxonomy.md");
    for needle in ["execution", "framework", "compiler", "incidental"] {
        assert!(
            adr.contains(needle),
            "ADR-020 must mention `{needle}` as part of the taxonomy"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("hotspot taxonomy"),
        "reference index must mention the profiling hotspot taxonomy"
    );
}
