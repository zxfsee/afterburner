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
fn profiling_environment_snapshot_refresh_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "profiling_environment_snapshot_refresh.schema.json",
    ))
    .expect("read profiling environment snapshot refresh schema fixture");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse profiling environment snapshot refresh schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/profiling-environment-snapshot-refresh/v1")
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
        "previous_snapshot_path",
        "refreshed_snapshot_path",
        "profile_kind",
        "profiler",
        "previous_profiler_path",
        "refreshed_profiler_path",
        "previous_profiler_version",
        "refreshed_profiler_version",
        "captured_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "profile-refresh-environment-snapshot current_snapshot profiler_path captured_at_unix_ms:"
        ),
        "justfile must expose the profile-refresh-environment-snapshot workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("profiling_environment_snapshot_refresh.json"),
        "workflow reference must mention the profiling environment snapshot refresh receipt"
    );
    assert!(
        workflows.contains("just profile-refresh-environment-snapshot"),
        "workflow reference must mention the profiling environment snapshot refresh workflow"
    );
}

#[test]
fn profile_refresh_environment_snapshot_writes_snapshot_and_receipt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let previous_snapshot = tmp.path().join("profiling_environment_snapshot.json");
    fs::write(
        &previous_snapshot,
        r#"{"schema_version":"1","profile_kind":"infer","profiler":"cargo-flamegraph","profiler_path":"/tmp/old-profiler","profiler_version":"old-profiler 1.0.0","host_os":"macos","host_arch":"aarch64"}"#,
    )
    .expect("write previous snapshot");

    let profiler = tmp.path().join("fake-profiler");
    fs::write(&profiler, "#!/bin/sh\necho fake-profiler 2.0.0\n").expect("write fake profiler");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&profiler).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&profiler, permissions).expect("set permissions");
    }

    let out = tmp.path().join("refreshed_snapshot.json");
    let receipt = tmp
        .path()
        .join("profiling_environment_snapshot_refresh.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("profile")
        .arg("refresh-environment-snapshot")
        .arg("--current-snapshot")
        .arg(&previous_snapshot)
        .arg("--profiler-path")
        .arg(&profiler)
        .arg("--captured-at-unix-ms")
        .arg("1735689600000")
        .arg("--out")
        .arg(&out)
        .arg("--refresh-receipt")
        .arg(&receipt);
    cmd.assert().success();

    let refreshed_text = fs::read_to_string(&out).expect("read refreshed snapshot");
    let refreshed: Value = serde_json::from_str(&refreshed_text).expect("parse refreshed snapshot");
    assert_eq!(
        refreshed,
        serde_json::json!({
            "schema_version": "1",
            "profile_kind": "infer",
            "profiler": "cargo-flamegraph",
            "profiler_path": profiler.display().to_string(),
            "profiler_version": "fake-profiler 2.0.0",
            "host_os": std::env::consts::OS,
            "host_arch": std::env::consts::ARCH
        })
    );

    let receipt_text = fs::read_to_string(&receipt).expect("read refresh receipt");
    let mut receipt_value: Value =
        serde_json::from_str(&receipt_text).expect("parse refresh receipt");
    let object = receipt_value
        .as_object_mut()
        .expect("refresh receipt must be object");
    object.insert(
        "previous_snapshot_path".to_string(),
        Value::from("<previous-snapshot>"),
    );
    object.insert(
        "refreshed_snapshot_path".to_string(),
        Value::from("<refreshed-snapshot>"),
    );
    object.insert(
        "previous_profiler_path".to_string(),
        Value::from("/tmp/old-profiler"),
    );
    object.insert(
        "refreshed_profiler_path".to_string(),
        Value::from("<profiler>"),
    );
    assert_eq!(
        receipt_value,
        serde_json::json!({
            "schema_version": "1",
            "previous_snapshot_path": "<previous-snapshot>",
            "refreshed_snapshot_path": "<refreshed-snapshot>",
            "profile_kind": "infer",
            "profiler": "cargo-flamegraph",
            "previous_profiler_path": "/tmp/old-profiler",
            "refreshed_profiler_path": "<profiler>",
            "previous_profiler_version": "old-profiler 1.0.0",
            "refreshed_profiler_version": "fake-profiler 2.0.0",
            "captured_at_unix_ms": 1735689600000i64
        })
    );
}
