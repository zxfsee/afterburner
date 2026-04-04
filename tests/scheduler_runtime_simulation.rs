use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

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
fn scheduler_runtime_simulation_schema_and_workflow_are_explicit() {
    let scenario_schema_text = fs::read_to_string(fixture_path(
        "scheduler_runtime_simulation_scenario.schema.json",
    ))
    .expect("read simulation scenario schema");
    let scenario_schema: Value =
        serde_json::from_str(&scenario_schema_text).expect("parse simulation scenario schema");
    assert_eq!(
        scenario_schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/scheduler-runtime-simulation-scenario/v1")
    );

    let report_schema_text = fs::read_to_string(fixture_path(
        "scheduler_runtime_simulation_report.schema.json",
    ))
    .expect("read simulation report schema");
    let report_schema: Value =
        serde_json::from_str(&report_schema_text).expect("parse simulation report schema");
    assert_eq!(
        report_schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/scheduler-runtime-simulation-report/v1")
    );
    let required = report_schema
        .get("required")
        .and_then(Value::as_array)
        .expect("report schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("required entries must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "scenario_name",
        "scenario_path",
        "tick_count",
        "scheduler_lifecycle",
        "lease_transitions",
        "layout_feasibility",
        "checkpoint_restart",
        "artifact_emission",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("scheduler-runtime-simulate scenario:"),
        "justfile must expose the scheduler-runtime-simulate workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("scheduler_runtime_simulation_report.json"),
        "workflow reference must mention the simulation report artifact"
    );
    assert!(
        workflows.contains("just scheduler-runtime-simulate"),
        "workflow reference must mention the simulation workflow"
    );

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("scheduler_runtime_simulation_report.json"),
        "reference index must mention the simulation report artifact"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("control-logic simulation harness"),
        "architecture must describe the simulation harness as a control-logic harness"
    );
}

#[test]
fn scheduler_runtime_simulation_writes_report_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("scheduler_runtime_simulation_report.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("simulate-scheduler-runtime")
        .arg("--scenario")
        .arg(fixture_path(
            "scheduler_runtime_simulation_scenario.example.json",
        ))
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let report_text = fs::read_to_string(&out).expect("read simulation report");
    let mut report: Value = serde_json::from_str(&report_text).expect("parse simulation report");
    report
        .as_object_mut()
        .expect("report must be object")
        .insert("scenario_path".to_string(), Value::from("<scenario>"));

    let expected_text = fs::read_to_string(fixture_path(
        "scheduler_runtime_simulation_report.example.json",
    ))
    .expect("read simulation report example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse simulation report example");
    expected
        .as_object_mut()
        .expect("expected report must be object")
        .insert("scenario_path".to_string(), Value::from("<scenario>"));
    assert_eq!(report, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("scheduler_runtime_simulation_report_written")
        })
        .unwrap_or_else(|| panic!("missing simulation report event in stderr: {stderr}"));
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("report_path".to_string(), Value::from("<report>"));
    let report_obj = fields
        .get_mut("report")
        .and_then(Value::as_object_mut)
        .expect("event report must be object");
    report_obj.insert("scenario_path".to_string(), Value::from("<scenario>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "debug_cli",
            "event": "scheduler_runtime_simulation_report_written",
            "fields": {
                "report_path": "<report>",
                "report": expected
            }
        })
    );
}
