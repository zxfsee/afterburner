use std::collections::BTreeSet;
use std::fs;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn distributed_runtime_layout_feasibility_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_runtime_layout_feasibility.schema.json",
    ))
    .expect("read feasibility schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/distributed-runtime-layout-feasibility/v1")
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
        "node_count",
        "gpus_per_node",
        "world_size",
        "parallelism",
        "feasible",
        "checks",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn distributed_runtime_layout_feasibility_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-054"),
        "architecture decisions index must link ADR-054"
    );
    assert!(
        architecture.contains("feasibility artifact"),
        "architecture must mention the layout feasibility artifact"
    );

    let adr = repo_file("docs/adr/054-distributed-runtime-layout-feasibility.md");
    for needle in [
        "distributed_runtime_layout_feasibility.schema.json",
        "node_count",
        "gpus_per_node",
        "world_size",
        "dp",
        "tp",
        "pp",
        "sp_cp",
        "ep",
        "gas",
        "microbatch_size",
        "zero_stage",
        "feasible",
        "checks",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-054 must mention `{needle}` as part of the feasibility contract"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("feasibility artifact"),
        "reference index must mention the distributed layout feasibility stance"
    );
}
