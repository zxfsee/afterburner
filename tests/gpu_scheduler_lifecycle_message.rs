use std::collections::BTreeSet;
use std::fs;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn gpu_scheduler_lifecycle_message_schema_is_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("gpu_scheduler_lifecycle_message.schema.json"))
            .expect("read gpu scheduler lifecycle schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/gpu-scheduler-lifecycle-message/v1")
    );

    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("schema.required must be an array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required entries must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = ["schema_version", "direction", "kind", "job_id", "lease_id"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let start_text = fs::read_to_string(fixture_path("gpu_scheduler_start_message.example.json"))
        .expect("read start example");
    let start: serde_json::Value = serde_json::from_str(&start_text).expect("parse start example");
    assert_eq!(start.get("kind").and_then(|v| v.as_str()), Some("START"));
    assert!(
        start.get("resources").is_some(),
        "START must carry resources"
    );

    let checkpointed_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_checkpointed_message.example.json",
    ))
    .expect("read checkpointed example");
    let checkpointed: serde_json::Value =
        serde_json::from_str(&checkpointed_text).expect("parse checkpointed example");
    assert_eq!(
        checkpointed.get("kind").and_then(|v| v.as_str()),
        Some("CHECKPOINTED")
    );
}

#[test]
fn gpu_scheduler_boundary_and_lifecycle_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-045"),
        "architecture decisions index must link ADR-045"
    );
    for needle in [
        "START",
        "STOP",
        "KILL",
        "READY",
        "CHECKPOINTED",
        "FAILED",
        "HEARTBEAT",
        "lease_id",
        "rank_assignments",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the scheduler lifecycle contract"
        );
    }

    let adr = repo_file("docs/adr/045-gpu-scheduler-boundary-and-lifecycle.md");
    for needle in [
        "gpu_scheduler_lifecycle_message.schema.json",
        "`START`",
        "`STOP`",
        "`KILL`",
        "`READY`",
        "`CHECKPOINTED`",
        "`FAILED`",
        "`HEARTBEAT`",
        "`lease_id`",
        "`rank_assignments`",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-045 must mention `{needle}` as part of the lifecycle decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("lease-owned resources and rank assignments"),
        "reference index must document the scheduler lifecycle boundary"
    );
}
