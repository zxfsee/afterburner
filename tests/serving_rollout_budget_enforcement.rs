#![cfg(unix)]

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use burn::{backend::ndarray::NdArray, prelude::*, record::CompactRecorder};
use serde_json::Value;

type CpuBackend = NdArray<f32>;

fn build_runtime_model_artifact() -> (tempfile::TempDir, PathBuf) {
    let artifact_dir = tempfile::tempdir().expect("create model artifact tempdir");
    let weights_path = artifact_dir.path().join("model.mpk");

    let device = <CpuBackend as Backend>::Device::default();
    let model = afterburner::model::ModelConfig::new(10).init::<CpuBackend>(&device);
    model
        .save_file(&weights_path, &CompactRecorder::new())
        .expect("write model artifact");

    let checksum =
        afterburner::manifest::compute_sha256_hex(&weights_path).expect("compute model checksum");
    let manifest =
        afterburner::manifest::ArtifactManifest::for_current("model.mpk", "0.1.0", checksum);
    manifest
        .write_to_dir(artifact_dir.path())
        .expect("write model manifest");

    (artifact_dir, weights_path)
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn fixture_json(name: &str) -> Value {
    let text = fs::read_to_string(fixture_path(name)).expect("read json fixture");
    serde_json::from_str(&text).expect("parse json fixture")
}

fn reserve_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("listener addr").port();
    drop(listener);
    port
}

fn wait_for_healthz(port: u16, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
            let _ = stream.write_all(
                b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            );
            let mut buf = String::new();
            let _ = stream.read_to_string(&mut buf);
            if buf.contains("200 OK") {
                return;
            }
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("server did not become healthy within {:?}", timeout);
}

fn wait_for_exit_ok(child: &mut Child, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("try_wait child") {
            assert!(
                status.success(),
                "server exit status must be success: {status}"
            );
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    let _ = child.kill();
    panic!("server did not exit within {:?}", timeout);
}

fn infer_payload() -> String {
    let bytes = "0,".repeat((28usize * 28).saturating_sub(1)) + "0";
    format!("{{\"bytes\":[{}]}}", bytes)
}

fn post_infer(port: u16, observed_latency_ms_p99: u64, observed_error_ratio: f64) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect infer");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set read timeout");

    let payload = infer_payload();
    let request_header = format!(
        "POST /infer HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nX-Rollout-Observed-Latency-P99-Ms: {observed_latency_ms_p99}\r\nX-Rollout-Observed-Error-Ratio: {observed_error_ratio}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream
        .write_all(request_header.as_bytes())
        .expect("write headers");
    stream.write_all(payload.as_bytes()).expect("write body");
    stream.flush().expect("flush request");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read infer response");
    response
}

fn post_infer_without_rollout_headers(port: u16) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect infer");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set read timeout");

    let payload = infer_payload();
    let request_header = format!(
        "POST /infer HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream
        .write_all(request_header.as_bytes())
        .expect("write headers");
    stream.write_all(payload.as_bytes()).expect("write body");
    stream.flush().expect("flush request");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read infer response");
    response
}

fn response_json_body(response: &str) -> Value {
    let (_, body) = response
        .split_once("\r\n\r\n")
        .expect("http response must include body separator");
    serde_json::from_str(body).expect("parse http response body json")
}

#[test]
fn serving_rollout_budget_enforces_latency_error_admission_policy() {
    let port = reserve_port();
    let (_artifact_dir, weights_path) = build_runtime_model_artifact();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(weights_path)
        .env("BACKEND", "cpu")
        .env("PORT", port.to_string())
        .env("HTTP_MAX_CONCURRENCY", "1")
        .env("HTTP_MAX_BATCH_SIZE", "16")
        .env("HTTP_MAX_REQUEST_BYTES", "300000")
        .env("HTTP_INFER_TIMEOUT_MS", "5000")
        .env(
            "HTTP_ROLLOUT_BUDGET_METADATA_JSON",
            r#"{"schema_version":"1","service":"mnist-http","latency_budget_ms_p99":100,"error_budget_ratio":0.05,"window":"5m"}"#,
        )
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn afterburner-http");
    let mut stderr = child.stderr.take().expect("take child stderr");

    wait_for_healthz(port, Duration::from_secs(20));

    let admitted = post_infer(port, 95, 0.04);
    assert!(
        admitted.contains("200 OK"),
        "under-budget request must be admitted: {admitted}"
    );
    let admitted_body = response_json_body(&admitted);
    let admitted_obj = admitted_body
        .as_object()
        .expect("admitted response body must be an object");
    let mut admitted_keys = admitted_obj.keys().cloned().collect::<Vec<_>>();
    admitted_keys.sort();
    assert_eq!(
        admitted_keys,
        vec!["batch_size".to_string(), "logits".to_string()],
        "admitted payload keys must remain stable"
    );

    let denied = post_infer(port, 140, 0.08);
    assert!(
        denied.contains("503 Service Unavailable"),
        "over-budget request must be denied: {denied}"
    );
    let denied_body = response_json_body(&denied);
    let expected_denied = fixture_json("http_error_rollout_budget_exceeded.json");
    assert_eq!(
        denied_body, expected_denied,
        "denied rollout payload must match fixture contract"
    );

    let pid = child.id().to_string();
    let kill_status = Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .expect("send SIGINT");
    assert!(kill_status.success(), "kill -INT must succeed");
    wait_for_exit_ok(&mut child, Duration::from_secs(5));

    let mut stderr_text = String::new();
    stderr
        .read_to_string(&mut stderr_text)
        .expect("read server stderr");
    let events = stderr_text
        .lines()
        .map(|line| serde_json::from_str(line).expect("parse observability event line"))
        .collect::<Vec<Value>>();
    let admission_events = events
        .iter()
        .filter(|event| {
            event.get("event").and_then(Value::as_str) == Some("rollout_budget_admission")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        admission_events.len(),
        2,
        "two infer requests must emit two rollout_budget_admission events"
    );

    let required_fields =
        fixture_json("serving_rollout_budget_admission_event_required_fields.json")
            .get("required")
            .and_then(Value::as_array)
            .expect("required fields fixture must contain array")
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .expect("required field names must be strings")
                    .to_string()
            })
            .collect::<Vec<_>>();
    let mut expected_keys = required_fields;
    expected_keys.sort();

    for event in &admission_events {
        assert_eq!(event.get("level").and_then(Value::as_str), Some("info"));
        assert_eq!(
            event.get("source").and_then(Value::as_str),
            Some("http_adapter")
        );
        let fields = event
            .get("fields")
            .and_then(Value::as_object)
            .expect("admission event fields must be object");
        let mut keys = fields.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        assert_eq!(
            keys, expected_keys,
            "admission event field keys must match fixture"
        );
        assert!(
            fields
                .get("request_id")
                .and_then(Value::as_str)
                .is_some_and(|request_id| !request_id.is_empty()),
            "request_id must be a non-empty string"
        );
    }

    let admitted_event = admission_events
        .iter()
        .copied()
        .find(|event| {
            event
                .get("fields")
                .and_then(Value::as_object)
                .and_then(|fields| fields.get("admitted"))
                .and_then(Value::as_bool)
                == Some(true)
        })
        .expect("missing admitted rollout event");
    let admitted_fields = admitted_event
        .get("fields")
        .and_then(Value::as_object)
        .expect("admitted event fields must be object");
    assert_eq!(
        admitted_fields.get("service").and_then(Value::as_str),
        Some("mnist-http")
    );
    assert_eq!(
        admitted_fields.get("window").and_then(Value::as_str),
        Some("5m")
    );
    assert_eq!(
        admitted_fields
            .get("latency_budget_ms_p99")
            .and_then(Value::as_u64),
        Some(100)
    );
    assert_eq!(
        admitted_fields
            .get("observed_latency_ms_p99")
            .and_then(Value::as_u64),
        Some(95)
    );
    assert_eq!(
        admitted_fields
            .get("error_budget_ratio")
            .and_then(Value::as_f64),
        Some(0.05)
    );
    assert_eq!(
        admitted_fields
            .get("observed_error_ratio")
            .and_then(Value::as_f64),
        Some(0.04)
    );

    let denied_event = admission_events
        .iter()
        .copied()
        .find(|event| {
            event
                .get("fields")
                .and_then(Value::as_object)
                .and_then(|fields| fields.get("admitted"))
                .and_then(Value::as_bool)
                == Some(false)
        })
        .expect("missing denied rollout event");
    let denied_fields = denied_event
        .get("fields")
        .and_then(Value::as_object)
        .expect("denied event fields must be object");
    assert_eq!(
        denied_fields
            .get("observed_latency_ms_p99")
            .and_then(Value::as_u64),
        Some(140)
    );
    assert_eq!(
        denied_fields
            .get("observed_error_ratio")
            .and_then(Value::as_f64),
        Some(0.08)
    );
}

#[test]
fn serving_rollout_budget_schema_typed_metadata_enables_admission_policy() {
    let port = reserve_port();
    let (_artifact_dir, weights_path) = build_runtime_model_artifact();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(weights_path)
        .env("BACKEND", "cpu")
        .env("PORT", port.to_string())
        .env("HTTP_MAX_CONCURRENCY", "1")
        .env("HTTP_MAX_BATCH_SIZE", "16")
        .env("HTTP_MAX_REQUEST_BYTES", "300000")
        .env("HTTP_INFER_TIMEOUT_MS", "5000")
        .env(
            "HTTP_ROLLOUT_BUDGET_METADATA_JSON",
            r#"{"schema_version":"1","service":"  ","latency_budget_ms_p99":100,"error_budget_ratio":0.05,"window":"  "}"#,
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn afterburner-http");

    wait_for_healthz(port, Duration::from_secs(20));

    let denied = post_infer_without_rollout_headers(port);
    assert!(
        denied.contains("400 Bad Request"),
        "missing rollout headers must be rejected when policy is enabled: {denied}"
    );
    assert!(
        denied.contains("\"kind\":\"invalid_rollout_observation\""),
        "error payload must identify rollout observation failure: {denied}"
    );

    let pid = child.id().to_string();
    let kill_status = Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .expect("send SIGINT");
    assert!(kill_status.success(), "kill -INT must succeed");
    wait_for_exit_ok(&mut child, Duration::from_secs(5));
}
