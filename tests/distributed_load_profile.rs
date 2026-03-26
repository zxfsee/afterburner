#![cfg(unix)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

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

fn reserve_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("listener addr").port();
    drop(listener);
    port
}

fn start_stub_server(port: u16, expected_requests: usize) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind stub server");
        let response_body = br#"{"batch_size":1,"logits":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
            response_body.len()
        );
        for _ in 0..expected_requests {
            let (mut stream, _) = listener.accept().expect("accept client");
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf).expect("read request");
            thread::sleep(Duration::from_millis(10));
            stream
                .write_all(response.as_bytes())
                .expect("write response");
            stream
                .write_all(response_body)
                .expect("write response body");
            stream.flush().expect("flush response");
        }
    })
}

#[test]
fn distributed_load_profile_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("distributed_load_profile.schema.json"))
        .expect("read distributed load profile schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse distributed load profile schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-load-profile/v1")
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
        "target_addr",
        "traceparent",
        "trace_id",
        "request_count",
        "concurrency",
        "success_count",
        "error_count",
        "error_ratio",
        "latency_ms_p50",
        "latency_ms_p95",
        "latency_ms_p99",
        "throughput_requests_per_sec",
        "latency_budget_ms_p99",
        "error_budget_ratio",
        "passed",
        "worker_distribution",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "distributed-load-profile addr requests concurrency latency_budget_ms_p99 error_budget_ratio:"
        ),
        "justfile must expose the distributed-load-profile workflow"
    );

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("distributed_load_profile.json"),
        "workflow reference must mention the distributed load profile artifact"
    );
    assert!(
        readme.contains("just distributed-load-profile"),
        "workflow reference must mention the distributed load profile workflow"
    );
}

#[test]
fn distributed_load_profile_writes_profile_and_event() {
    let port = reserve_port();
    let _server = start_stub_server(port, 6);
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out = out_dir.path().join("distributed_load_profile.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("load-profile")
        .arg("--addr")
        .arg(format!("127.0.0.1:{port}"))
        .arg("--requests")
        .arg("6")
        .arg("--concurrency")
        .arg("2")
        .arg("--latency-budget-ms-p99")
        .arg("500")
        .arg("--error-budget-ratio")
        .arg("0.1")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read distributed load profile");
    let profile: Value = serde_json::from_str(&text).expect("parse distributed load profile");
    let object = profile
        .as_object()
        .expect("distributed load profile must be object");
    assert_eq!(
        object.get("schema_version").and_then(Value::as_str),
        Some("1")
    );
    assert_eq!(
        object.get("target_addr").and_then(Value::as_str),
        Some(format!("127.0.0.1:{port}").as_str())
    );
    let traceparent = object
        .get("traceparent")
        .and_then(Value::as_str)
        .expect("traceparent must be present");
    assert!(
        traceparent.starts_with("00-"),
        "traceparent must use w3c shape"
    );
    assert!(
        object
            .get("trace_id")
            .and_then(Value::as_str)
            .is_some_and(|value| value.len() == 32),
        "trace_id must be present"
    );
    assert_eq!(object.get("request_count").and_then(Value::as_i64), Some(6));
    assert_eq!(object.get("concurrency").and_then(Value::as_i64), Some(2));
    assert_eq!(object.get("success_count").and_then(Value::as_i64), Some(6));
    assert_eq!(object.get("error_count").and_then(Value::as_i64), Some(0));
    assert_eq!(object.get("error_ratio").and_then(Value::as_f64), Some(0.0));
    assert_eq!(
        object.get("latency_budget_ms_p99").and_then(Value::as_f64),
        Some(500.0)
    );
    assert_eq!(
        object.get("error_budget_ratio").and_then(Value::as_f64),
        Some(0.1)
    );
    assert_eq!(object.get("passed").and_then(Value::as_bool), Some(true));
    assert!(
        object
            .get("throughput_requests_per_sec")
            .and_then(Value::as_f64)
            .is_some_and(|value| value > 0.0),
        "throughput must be positive"
    );
    for key in ["latency_ms_p50", "latency_ms_p95", "latency_ms_p99"] {
        assert!(
            object
                .get(key)
                .and_then(Value::as_f64)
                .is_some_and(|value| value >= 0.0),
            "{key} must be present and non-negative"
        );
    }
    let distribution = object
        .get("worker_distribution")
        .and_then(Value::as_array)
        .expect("worker_distribution must be an array");
    assert_eq!(distribution.len(), 2);
    let total_requests: i64 = distribution
        .iter()
        .map(|worker| {
            worker
                .get("request_count")
                .and_then(Value::as_i64)
                .expect("worker request_count must be integer")
        })
        .sum();
    assert_eq!(total_requests, 6);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "distributed_load_profile_written");
    let fields = event
        .get("fields")
        .and_then(Value::as_object)
        .expect("event fields must be object");
    assert_eq!(
        fields.get("profile_path").and_then(Value::as_str),
        Some(out.display().to_string().as_str())
    );
    assert_eq!(
        fields
            .get("profile")
            .and_then(Value::as_object)
            .and_then(|profile| profile.get("request_count"))
            .and_then(Value::as_i64),
        Some(6)
    );
    assert_eq!(
        fields.get("traceparent").and_then(Value::as_str),
        Some(traceparent)
    );
    assert_eq!(
        fields.get("trace_id").and_then(Value::as_str),
        object.get("trace_id").and_then(Value::as_str)
    );
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}
