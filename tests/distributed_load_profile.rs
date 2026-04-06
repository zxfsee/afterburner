#![cfg(unix)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

fn start_stub_server(
    expected_requests: usize,
    timeout: Duration,
) -> Option<(u16, thread::JoinHandle<()>)> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(err) if err.kind() == ErrorKind::PermissionDenied => return None,
        Err(err) => panic!("bind ephemeral port: {err}"),
    };
    let port = listener.local_addr().expect("listener addr").port();
    let (ready_tx, ready_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");
        ready_tx.send(()).expect("send stub-server ready signal");
        let response_body = br#"{"batch_size":1,"logits":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
            response_body.len()
        );
        let deadline = Instant::now() + timeout;
        let mut served = 0usize;
        while served < expected_requests {
            if Instant::now() >= deadline {
                panic!("stub server timed out after serving {served}/{expected_requests} requests");
            }
            let (mut stream, _) = match listener.accept() {
                Ok(pair) => pair,
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(err) => panic!("accept client: {err}"),
            };
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
            served += 1;
        }
    });
    ready_rx
        .recv_timeout(timeout)
        .expect("receive stub-server ready signal within timeout");
    Some((port, handle))
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

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("distributed_load_profile.json"),
        "workflow reference must mention the distributed load profile artifact"
    );
    assert!(
        workflows.contains("just distributed-load-profile"),
        "workflow reference must mention the distributed load profile workflow"
    );
}

#[test]
fn distributed_load_profile_writes_profile_and_event() {
    let Some((port, server)) = start_stub_server(6, Duration::from_secs(5)) else {
        return;
    };
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
    server.join().expect("join stub server thread");
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}
