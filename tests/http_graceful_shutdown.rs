#![cfg(unix)]

use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

#[path = "fixture_support.rs"]
mod fixture_support;

fn reserve_port() -> Option<u16> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(err) if err.kind() == ErrorKind::PermissionDenied => return None,
        Err(err) => panic!("bind ephemeral port: {err}"),
    };
    let port = listener.local_addr().expect("listener addr").port();
    drop(listener);
    Some(port)
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

fn wait_for_log(logs: &Arc<Mutex<Vec<String>>>, needle: &str, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if logs
            .lock()
            .expect("lock logs")
            .iter()
            .any(|line| line.contains(needle))
        {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("log line not observed within {:?}: {}", timeout, needle);
}

fn infer_batch_payload(batch_size: usize) -> String {
    let single = "0,".repeat((28usize * 28).saturating_sub(1)) + "0";
    let item = format!("{{\"bytes\":[{}]}}", single);
    let mut batch = String::new();
    for i in 0..batch_size {
        if i > 0 {
            batch.push(',');
        }
        batch.push_str(&item);
    }
    format!("{{\"batch\":[{}]}}", batch)
}

fn infer_single_payload() -> String {
    let single = "0,".repeat((28usize * 28).saturating_sub(1)) + "0";
    format!("{{\"bytes\":[{}]}}", single)
}

fn infer_error_artifact_not_found_fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("infer_error_artifact_not_found.fixture.json");
    let text = std::fs::read_to_string(path).expect("read infer error fixture");
    serde_json::from_str(&text).expect("parse infer error fixture")
}

fn manifest_fixture_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("manifest.toml");
    std::fs::read_to_string(path).expect("read manifest fixture")
}

fn json_fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    let text = std::fs::read_to_string(path).expect("read json fixture");
    serde_json::from_str(&text).expect("parse json fixture")
}

fn infer_success_envelope_fixture(name: &str) -> Value {
    json_fixture(name)
}

fn assert_success_payload_matches_fixture(payload: &Value, fixture_name: &str) {
    let fixture = infer_success_envelope_fixture(fixture_name);
    let fixture_obj = fixture
        .as_object()
        .expect("success envelope fixture must be an object");
    let required_top_level_keys = fixture_obj
        .get("required_top_level_keys")
        .and_then(Value::as_array)
        .expect("fixture must define required_top_level_keys array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required_top_level_keys values must be strings")
                .to_string()
        })
        .collect::<Vec<_>>();
    let expected_batch_size = fixture_obj
        .get("batch_size")
        .and_then(Value::as_u64)
        .expect("fixture batch_size must be u64");
    let expected_row_width = fixture_obj
        .get("row_width")
        .and_then(Value::as_u64)
        .expect("fixture row_width must be u64");

    let payload_obj = payload
        .as_object()
        .expect("success response envelope must be a json object");
    let mut keys = payload_obj.keys().cloned().collect::<Vec<_>>();
    let mut expected_keys = required_top_level_keys.clone();
    keys.sort();
    expected_keys.sort();
    assert_eq!(keys, expected_keys, "success envelope fields drifted");

    let batch_size = payload_obj
        .get("batch_size")
        .and_then(Value::as_u64)
        .expect("batch_size must be numeric");
    assert_eq!(
        batch_size, expected_batch_size,
        "batch_size must match fixture contract"
    );
    let logits = payload_obj
        .get("logits")
        .and_then(Value::as_array)
        .expect("logits must be an array");
    assert_eq!(
        logits.len() as u64,
        expected_batch_size,
        "logits outer length must match batch_size"
    );
    for (idx, row) in logits.iter().enumerate() {
        let row = row
            .as_array()
            .unwrap_or_else(|| panic!("logits[{idx}] must be an array"));
        assert_eq!(
            row.len() as u64,
            expected_row_width,
            "logits[{idx}] width must match fixture"
        );
        for (value_idx, value) in row.iter().enumerate() {
            let number = value
                .as_f64()
                .unwrap_or_else(|| panic!("logits[{idx}][{value_idx}] must be numeric"));
            assert!(
                number.is_finite(),
                "logits[{idx}][{value_idx}] must be finite"
            );
        }
    }
}

fn normalize_infer_error_line(line: &str) -> Value {
    let mut value: Value = serde_json::from_str(line).expect("parse infer error event");
    let object = value
        .as_object_mut()
        .expect("infer error event must be an object");
    object.insert("ts_ms".to_string(), json!(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("infer error event fields must be an object");
    fields.insert("artifact".to_string(), json!("does-not-exist.mpk"));
    value
}

fn stderr_event_from_logs(logs: &Arc<Mutex<Vec<String>>>, event_name: &str) -> Value {
    logs.lock()
        .expect("lock logs")
        .iter()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in server logs"))
}

fn normalize_http_infer_done_event(event: &Value) -> Value {
    let mut normalized = event.clone();
    let object = normalized
        .as_object_mut()
        .expect("http infer_done event must be an object");
    object.insert("ts_ms".to_string(), json!(0));

    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("http infer_done fields must be an object");
    fields.insert("request_id".to_string(), json!("<request_id>"));
    fields.insert(
        "traceparent".to_string(),
        json!("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
    );
    fields.insert(
        "trace_id".to_string(),
        json!("4bf92f3577b34da6a3ce929d0e0e4736"),
    );

    let duration_ms = fields
        .get("duration_ms")
        .and_then(Value::as_u64)
        .expect("duration_ms must be numeric");
    assert!(duration_ms >= 1, "duration_ms must be positive");
    fields.insert("duration_ms".to_string(), json!(1));

    normalized
}

fn assert_precision_fields(fields: &serde_json::Map<String, Value>) {
    let precision = fields
        .get("precision")
        .and_then(Value::as_object)
        .expect("precision fields must be present");
    assert_eq!(
        precision.get("weights_dtype").and_then(Value::as_str),
        Some("f32")
    );
    assert_eq!(
        precision.get("activation_dtype").and_then(Value::as_str),
        Some("f32")
    );
    assert_eq!(
        precision.get("quantization").and_then(Value::as_str),
        Some("none")
    );
}

#[test]
fn in_flight_infer_completes_on_sigint() {
    let Some(port) = reserve_port() else {
        return;
    };
    let (_artifact_dir, weights_path) = fixture_support::build_runtime_model_artifact();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(&weights_path)
        .env("BACKEND", "cpu")
        .env("PORT", port.to_string())
        .env("HTTP_MAX_CONCURRENCY", "1")
        .env("HTTP_MAX_BATCH_SIZE", "16")
        .env("HTTP_MAX_REQUEST_BYTES", "300000")
        .env("HTTP_INFER_TIMEOUT_MS", "10000")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn afterburner-http");
    let stderr = child.stderr.take().expect("take child stderr");
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let logs_reader = Arc::clone(&logs);
    let stderr_thread = thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in std::io::BufRead::lines(reader).map_while(Result::ok) {
            logs_reader.lock().expect("lock logs").push(line);
        }
    });

    wait_for_healthz(port, Duration::from_secs(20));

    let infer_handle = thread::spawn(move || {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect infer");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");

        let payload = infer_batch_payload(16);
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
    });

    wait_for_log(
        &logs,
        r#""event":"http_request","fields":{"method":"POST","path":"/infer""#,
        Duration::from_secs(5),
    );

    let pid = child.id().to_string();
    let kill_status = Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .expect("send SIGINT");
    assert!(kill_status.success(), "kill -INT must succeed");

    let response = infer_handle.join().expect("join infer request thread");
    assert!(
        response.contains("200 OK"),
        "response must be 200: {response}"
    );
    assert!(
        response.contains("\"batch_size\":16"),
        "response must contain batch payload: {response}"
    );
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("response must include headers and body");
    let payload: Value = serde_json::from_str(body).expect("response body must be json");
    assert_success_payload_matches_fixture(&payload, "http_infer_success_envelope.fixture.json");

    wait_for_exit_ok(&mut child, Duration::from_secs(5));
    stderr_thread.join().expect("join stderr reader");
}

#[test]
fn single_item_infer_response_matches_single_success_fixture() {
    let Some(port) = reserve_port() else {
        return;
    };
    let (_artifact_dir, weights_path) = fixture_support::build_runtime_model_artifact();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(&weights_path)
        .env("BACKEND", "cpu")
        .env("PORT", port.to_string())
        .env("HTTP_MAX_CONCURRENCY", "1")
        .env("HTTP_MAX_BATCH_SIZE", "16")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn afterburner-http");
    let stderr = child.stderr.take().expect("take child stderr");
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let logs_reader = Arc::clone(&logs);
    let stderr_thread = thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in std::io::BufRead::lines(reader).map_while(Result::ok) {
            logs_reader.lock().expect("lock logs").push(line);
        }
    });

    wait_for_healthz(port, Duration::from_secs(20));

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect infer");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("set read timeout");

    let payload = infer_single_payload();
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
    assert!(
        response.contains("200 OK"),
        "response must be 200: {response}"
    );
    assert!(
        response.contains("\"batch_size\":1"),
        "response must contain single payload batch_size: {response}"
    );

    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("response must include headers and body");
    let payload: Value = serde_json::from_str(body).expect("response body must be json");
    assert_success_payload_matches_fixture(
        &payload,
        "http_infer_single_success_envelope.fixture.json",
    );

    let pid = child.id().to_string();
    let kill_status = Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .expect("send SIGINT");
    assert!(kill_status.success(), "kill -INT must succeed");
    wait_for_exit_ok(&mut child, Duration::from_secs(5));
    stderr_thread.join().expect("join stderr reader");
}

#[test]
fn single_item_http_infer_done_event_matches_fixture() {
    let Some(port) = reserve_port() else {
        return;
    };
    let (_artifact_dir, weights_path) = fixture_support::build_runtime_model_artifact();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(&weights_path)
        .env("BACKEND", "cpu")
        .env("PORT", port.to_string())
        .env("HTTP_MAX_CONCURRENCY", "1")
        .env("HTTP_MAX_BATCH_SIZE", "16")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn afterburner-http");
    let stderr = child.stderr.take().expect("take child stderr");
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let logs_reader = Arc::clone(&logs);
    let stderr_thread = thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in std::io::BufRead::lines(reader).map_while(Result::ok) {
            logs_reader.lock().expect("lock logs").push(line);
        }
    });

    wait_for_healthz(port, Duration::from_secs(20));

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect infer");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("set read timeout");

    let payload = infer_single_payload();
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
    assert!(
        response.contains("200 OK"),
        "response must be 200: {response}"
    );

    wait_for_log(&logs, r#""event":"infer_done""#, Duration::from_secs(5));
    let artifact_load_ok = stderr_event_from_logs(&logs, "artifact_load_ok");
    assert_precision_fields(
        artifact_load_ok
            .get("fields")
            .and_then(Value::as_object)
            .expect("http artifact_load_ok fields must be an object"),
    );
    let infer_done = stderr_event_from_logs(&logs, "infer_done");
    assert_precision_fields(
        infer_done
            .get("fields")
            .and_then(Value::as_object)
            .expect("http infer_done fields must be an object"),
    );
    let normalized = normalize_http_infer_done_event(&infer_done);
    let expected = json_fixture("http_infer_done_event.fixture.json");
    assert_eq!(
        normalized, expected,
        "http infer_done event must match the fixture-backed contract"
    );

    let pid = child.id().to_string();
    let kill_status = Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .expect("send SIGINT");
    assert!(kill_status.success(), "kill -INT must succeed");
    wait_for_exit_ok(&mut child, Duration::from_secs(5));
    stderr_thread.join().expect("join stderr reader");
}

#[test]
fn startup_missing_artifact_infer_error_matches_fixture() {
    let Some(port) = reserve_port() else {
        return;
    };
    let output = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg("does-not-exist.mpk")
        .env("BACKEND", "cpu")
        .env("PORT", port.to_string())
        .output()
        .expect("run afterburner-http with missing artifact");

    assert_eq!(
        output.status.code(),
        Some(2),
        "missing artifact must fail fast with exit code 2"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr must be utf-8");
    let infer_error_line = stderr
        .lines()
        .find(|line| line.contains(r#""event":"infer_error""#))
        .expect("infer_error event line must be emitted");

    let actual = normalize_infer_error_line(infer_error_line);
    let expected = infer_error_artifact_not_found_fixture();
    assert_eq!(actual, expected, "infer_error event envelope drifted");
}

#[test]
fn startup_quantized_manifest_fails_fast_with_manifest_invalid_field() {
    let Some(port) = reserve_port() else {
        return;
    };
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    let weights = dir.join("model.mpk");
    std::fs::write(&weights, "").expect("write weights");
    let checksum = afterburner::manifest::compute_sha256_hex(&weights).expect("checksum");
    let manifest = manifest_fixture_text()
        .replace(
            r#"artifact_sha256 = "b3b57b1fea16e57389145b129a3330998bf3c3bc1e412c46779b3e1827fdd55b""#,
            &format!(r#"artifact_sha256 = "{checksum}""#),
        )
        .replace(r#"quantization = "none""#, r#"quantization = "int8""#);
    std::fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(&weights)
        .env("BACKEND", "cpu")
        .env("PORT", port.to_string())
        .output()
        .expect("run afterburner-http with unsupported quantized artifact");

    assert_eq!(
        output.status.code(),
        Some(2),
        "unsupported quantized artifact must fail fast with exit code 2"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr must be utf-8");
    let infer_error_line = stderr
        .lines()
        .find(|line| line.contains(r#""event":"infer_error""#))
        .expect("infer_error event line must be emitted");

    let actual: Value = serde_json::from_str(infer_error_line).expect("parse infer error event");
    assert_eq!(
        actual.get("source").and_then(Value::as_str),
        Some("http_adapter")
    );
    assert_eq!(
        actual.get("event").and_then(Value::as_str),
        Some("infer_error")
    );
    let fields = actual
        .get("fields")
        .and_then(Value::as_object)
        .expect("infer error event fields must be object");
    assert_eq!(
        fields.get("kind").and_then(Value::as_str),
        Some("manifest_invalid_field")
    );
    assert_eq!(
        fields.get("field").and_then(Value::as_str),
        Some("precision.quantization")
    );
}
