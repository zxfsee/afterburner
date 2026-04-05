use std::collections::BTreeSet;
use std::fs;

use afterburner::observability::event_line;
use serde_json::json;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn event_line_matches_required_envelope_fixture_keys() {
    let fixture = fs::read_to_string(fixture_path(
        "observability_envelope_required_keys.fixture.json",
    ))
    .expect("read envelope fixture");
    let fixture: serde_json::Value =
        serde_json::from_str(&fixture).expect("parse envelope fixture");
    let required = fixture
        .get("required")
        .and_then(|v| v.as_array())
        .expect("fixture.required must be an array");
    let required_keys = required
        .iter()
        .map(|v| {
            v.as_str()
                .expect("fixture required keys must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    let line = event_line(
        "info",
        "infer_cli",
        "infer_done",
        json!({ "elapsed_ms": 1 }),
    );
    let event: serde_json::Value = serde_json::from_str(&line).expect("parse event line");
    let obj = event
        .as_object()
        .expect("event line must be a top-level JSON object");

    let actual_keys = obj.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        actual_keys, required_keys,
        "event envelope keys must match fixture"
    );

    assert!(
        event.get("ts_ms").and_then(|v| v.as_u64()).is_some(),
        "ts_ms must be a numeric unix timestamp in milliseconds"
    );
    assert!(
        event.get("fields").and_then(|v| v.as_object()).is_some(),
        "fields must be an object"
    );
}
