use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn profiling_environment_snapshot_schema_and_workflow_are_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("profiling_environment_snapshot.schema.json"))
            .expect("read profiling environment snapshot schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse profiling environment snapshot schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/profiling-environment-snapshot/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "profile_kind",
        "profiler",
        "profiler_path",
        "profiler_version",
        "host_os",
        "host_arch",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("profile-environment-snapshot:"),
        "justfile must expose the profile-environment-snapshot workflow"
    );
    assert!(
        justfile.contains("cargo run --locked --bin afterburner -- profile infer"),
        "profile-infer workflow must route through the native profile infer command"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("profiling_environment_snapshot.json"),
        "workflow reference must mention the profiling environment snapshot artifact"
    );
    assert!(
        workflows.contains("afterburner profile infer"),
        "workflow reference must mention the native profile infer workflow"
    );
}

#[test]
fn profile_environment_snapshot_writes_snapshot_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let profiler = tmp.path().join("fake-profiler");
    fs::write(&profiler, "#!/bin/sh\necho fake-profiler 1.2.3\n").expect("write fake profiler");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&profiler).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&profiler, permissions).expect("set permissions");
    }

    let out = tmp.path().join("profiling_environment_snapshot.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("profile")
        .arg("environment-snapshot")
        .arg("--profile-kind")
        .arg("infer")
        .arg("--profiler")
        .arg("cargo-flamegraph")
        .arg("--profiler-path")
        .arg(&profiler)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read profiling environment snapshot");
    let snapshot: Value =
        serde_json::from_str(&text).expect("parse profiling environment snapshot");
    assert_eq!(
        snapshot,
        serde_json::json!({
            "schema_version": "1",
            "profile_kind": "infer",
            "profiler": "cargo-flamegraph",
            "profiler_path": profiler.display().to_string(),
            "profiler_version": "fake-profiler 1.2.3",
            "host_os": std::env::consts::OS,
            "host_arch": std::env::consts::ARCH
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "profiling_environment_snapshot_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("snapshot_path".to_string(), Value::from("<snapshot>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "profiling_cli",
            "event": "profiling_environment_snapshot_written",
            "fields": {
                "snapshot_path": "<snapshot>",
                "snapshot": snapshot
            }
        })
    );
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}
