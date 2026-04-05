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
fn distributed_runtime_benchmark_harness_is_documented() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_runtime_benchmark_run.schema.json",
    ))
    .expect("read benchmark run schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-runtime-benchmark-run/v1")
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
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
        "artifact_version",
        "model_size",
        "node_count",
        "gpus_per_node",
        "world_size",
        "parallelism",
        "benchmark_config",
        "metrics",
        "runtime_settings",
        "environment_fingerprint",
        "benchmark_duration_ms",
        "benchmark_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-058"),
        "architecture decisions index must link ADR-058"
    );

    let adr = repo_file("docs/adr/058-distributed-runtime-benchmark-harness.md");
    for needle in [
        "distributed-runtime-benchmark",
        "benchmark configuration",
        "distributed_runtime_benchmark_run.json",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-058 must mention `{needle}` as part of the benchmark harness decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("distributed_runtime_benchmark_run.json"),
        "reference index must mention the benchmark run artifact"
    );
}

#[test]
fn distributed_runtime_benchmark_command_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("distributed_runtime_benchmark_run.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("profile")
        .arg("distributed-runtime-benchmark")
        .arg("--artifact-version")
        .arg("0.1.0")
        .arg("--model-size")
        .arg("8b")
        .arg("--node-count")
        .arg("1")
        .arg("--gpus-per-node")
        .arg("8")
        .arg("--world-size")
        .arg("8")
        .arg("--dp")
        .arg("8")
        .arg("--tp")
        .arg("1")
        .arg("--pp")
        .arg("1")
        .arg("--sp-cp")
        .arg("1")
        .arg("--ep")
        .arg("1")
        .arg("--gas")
        .arg("4")
        .arg("--microbatch-size")
        .arg("8")
        .arg("--zero-stage")
        .arg("0")
        .arg("--backend")
        .arg("wgpu")
        .arg("--system")
        .arg("aarch64-darwin")
        .arg("--gpu-model")
        .arg("apple-m4-max")
        .arg("--burn-version")
        .arg("0.20.1")
        .arg("--seed")
        .arg("42")
        .arg("--measurement-window-steps")
        .arg("8")
        .arg("--benchmark-command")
        .arg("true")
        .arg("--step-time-ms")
        .arg("187.5")
        .arg("--tokens-per-sec")
        .arg("20480")
        .arg("--mfu")
        .arg("0.42")
        .arg("--max-memory-bytes")
        .arg("21474836480")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read benchmark artifact");
    let mut artifact: Value = serde_json::from_str(&text).expect("parse artifact");
    artifact
        .as_object_mut()
        .expect("artifact must be object")
        .insert("benchmark_duration_ms".to_string(), Value::from(0));
    let expected_text = fs::read_to_string(fixture_path(
        "distributed_runtime_benchmark_run.example.json",
    ))
    .expect("read expected artifact");
    let expected: Value = serde_json::from_str(&expected_text).expect("parse expected artifact");
    assert_eq!(artifact, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("\"event\":\"distributed_runtime_benchmark_run_written\""),
        "benchmark command must emit a run-written event"
    );
}
