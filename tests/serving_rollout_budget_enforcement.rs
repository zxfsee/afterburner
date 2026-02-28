#![cfg(unix)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn fixture_model_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("inference")
        .join("0.1.0")
        .join("model.mpk")
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

#[test]
fn serving_rollout_budget_enforces_latency_error_admission_policy() {
    let port = reserve_port();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(fixture_model_path())
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
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn afterburner-http");

    wait_for_healthz(port, Duration::from_secs(20));

    let admitted = post_infer(port, 95, 0.04);
    assert!(
        admitted.contains("200 OK"),
        "under-budget request must be admitted: {admitted}"
    );
    assert!(
        admitted.contains("\"batch_size\":1"),
        "admitted response should include logits payload: {admitted}"
    );

    let denied = post_infer(port, 140, 0.08);
    assert!(
        denied.contains("503 Service Unavailable"),
        "over-budget request must be denied: {denied}"
    );
    assert!(
        denied.contains("\"kind\":\"rollout_budget_exceeded\""),
        "denied response must expose rollout budget policy error: {denied}"
    );

    let pid = child.id().to_string();
    let kill_status = Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .expect("send SIGINT");
    assert!(kill_status.success(), "kill -INT must succeed");
    wait_for_exit_ok(&mut child, Duration::from_secs(5));
}

#[test]
fn serving_rollout_budget_schema_typed_metadata_enables_admission_policy() {
    let port = reserve_port();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(fixture_model_path())
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
