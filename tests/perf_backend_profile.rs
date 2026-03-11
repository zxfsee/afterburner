use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn profile_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("eval")
        .join("backend_performance_profile.json")
}

#[test]
fn backend_performance_profile_has_objective_native_backend_criteria() {
    let text = fs::read_to_string(profile_path()).expect("read backend performance profile");
    let profile: Value = serde_json::from_str(&text).expect("parse backend performance profile");
    let profile = profile
        .as_object()
        .expect("backend performance profile must be a top-level object");

    assert_eq!(
        profile.get("schema_version").and_then(Value::as_str),
        Some("1")
    );
    assert_eq!(
        profile.get("recommended_backend").and_then(Value::as_str),
        Some("wgpu")
    );
    assert_eq!(
        profile.get("comparison_command").and_then(Value::as_str),
        Some("afterburner eval --seed 42 --batch-size 128 --max-batches 8")
    );

    let native_backend = profile
        .get("native_backend_adoption")
        .and_then(Value::as_object)
        .expect("native_backend_adoption must be an object");
    assert_eq!(
        native_backend
            .get("required_consecutive_regressions")
            .and_then(Value::as_u64),
        Some(2)
    );
    assert_eq!(
        native_backend
            .get("minimum_samples_per_profile")
            .and_then(Value::as_u64),
        Some(1024)
    );

    let triggers = native_backend
        .get("trigger_any")
        .and_then(Value::as_array)
        .expect("native_backend_adoption.trigger_any must be an array");
    assert!(
        !triggers.is_empty(),
        "native backend triggers must declare at least one objective rule"
    );

    let first_trigger = triggers[0]
        .as_object()
        .expect("native backend trigger entries must be objects");
    assert_eq!(
        first_trigger.get("metric").and_then(Value::as_str),
        Some("wgpu_eval_duration_ms")
    );
    assert_eq!(
        first_trigger.get("operator").and_then(Value::as_str),
        Some("gt")
    );
    assert_eq!(
        first_trigger
            .get("threshold_multiplier_vs_cpu")
            .and_then(Value::as_f64),
        Some(1.25)
    );
}
