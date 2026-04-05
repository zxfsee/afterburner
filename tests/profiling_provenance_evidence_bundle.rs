use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn profiling_provenance_evidence_bundle_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "profiling_provenance_evidence_bundle.schema.json",
    ))
    .expect("read profiling provenance evidence bundle schema fixture");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse profiling provenance evidence bundle schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/profiling-provenance-evidence-bundle/v1")
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
        "snapshot_path",
        "summary_path",
        "captured_at_unix_ms",
        "artifact_version",
        "profile_kind",
        "profiler",
        "backend",
        "evidence_count",
        "evidence_sources",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("profile-provenance-bundle snapshot summary captured_at_unix_ms:"),
        "justfile must expose the profile-provenance-bundle workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("profiling_provenance_evidence_bundle.json"),
        "workflow reference must mention the profiling provenance evidence bundle artifact"
    );
    assert!(
        workflows.contains("just profile-provenance-bundle"),
        "workflow reference must mention the profiling provenance evidence bundle workflow"
    );
}

#[test]
fn profiling_provenance_bundle_writes_bundle_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let snapshot = tmp.path().join("profiling_environment_snapshot.json");
    fs::write(
        &snapshot,
        r#"{
  "schema_version": "1",
  "profile_kind": "infer",
  "profiler": "cargo-flamegraph",
  "profiler_path": "/tmp/fake-profiler",
  "profiler_version": "fake-profiler 1.2.3",
  "host_os": "darwin",
  "host_arch": "aarch64"
}"#,
    )
    .expect("write profiling snapshot");

    let summary = tmp.path().join("profiling_hotspot_summary.json");
    fs::write(
        &summary,
        r#"{
  "schema_version": "1",
  "profile_kind": "infer",
  "profiler": "cargo-flamegraph",
  "weights_artifact": "artifacts/inference/0.1.0/model.mpk",
  "artifact_version": "0.1.0",
  "backend": "wgpu",
  "profile_command": "cargo flamegraph --dev --deterministic --bin afterburner -- infer artifacts/inference/0.1.0/model.mpk",
  "flamegraph_svg": "/tmp/fixture.svg",
  "total_samples": 42,
  "top_hotspots": [
    {"symbol":"afterburner::cmd_infer::run_infer","samples":30,"percent":71.43}
  ]
}"#,
    )
    .expect("write profiling summary");

    let out = tmp.path().join("profiling_provenance_evidence_bundle.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("profile")
        .arg("provenance-bundle")
        .arg("--snapshot")
        .arg(&snapshot)
        .arg("--summary")
        .arg(&summary)
        .arg("--captured-at-unix-ms")
        .arg("1735689600000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read profiling provenance bundle");
    let mut bundle: Value = serde_json::from_str(&text).expect("parse profiling provenance bundle");
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert("snapshot_path".to_string(), Value::from("<snapshot>"));
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert("summary_path".to_string(), Value::from("<summary>"));
    assert_eq!(
        bundle,
        serde_json::json!({
            "schema_version": "1",
            "snapshot_path": "<snapshot>",
            "summary_path": "<summary>",
            "captured_at_unix_ms": 1735689600000_u64,
            "artifact_version": "0.1.0",
            "profile_kind": "infer",
            "profiler": "cargo-flamegraph",
            "backend": "wgpu",
            "evidence_count": 2,
            "evidence_sources": [
                {
                    "artifact_path": snapshot.display().to_string(),
                    "artifact_role": "profiling_environment_snapshot",
                    "observed_at_unix_ms": 1735689600000_u64
                },
                {
                    "artifact_path": summary.display().to_string(),
                    "artifact_role": "profiling_hotspot_summary",
                    "observed_at_unix_ms": 1735689600000_u64
                }
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("profiling_provenance_evidence_bundle_written")
        })
        .unwrap_or_else(|| panic!("missing profiling provenance bundle event in stderr: {stderr}"));
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let bundle_obj = fields
        .get_mut("bundle")
        .and_then(Value::as_object_mut)
        .expect("event bundle must be an object");
    bundle_obj.insert("snapshot_path".to_string(), Value::from("<snapshot>"));
    bundle_obj.insert("summary_path".to_string(), Value::from("<summary>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "profiling_cli",
            "event": "profiling_provenance_evidence_bundle_written",
            "fields": {
                "bundle_path": "<bundle>",
                "bundle": bundle
            }
        })
    );
}
