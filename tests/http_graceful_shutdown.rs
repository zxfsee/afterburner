#![cfg(unix)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

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

fn infer_error_artifact_not_found_fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("infer_error_artifact_not_found.json");
    let text = std::fs::read_to_string(path).expect("read infer error fixture");
    serde_json::from_str(&text).expect("parse infer error fixture")
}

fn infer_success_envelope_fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("http_infer_success_envelope.json");
    let text = std::fs::read_to_string(path).expect("read infer success envelope fixture");
    serde_json::from_str(&text).expect("parse infer success envelope fixture")
}

fn assert_success_payload_matches_fixture(payload: &Value) {
    let fixture = infer_success_envelope_fixture();
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

#[test]
fn in_flight_infer_completes_on_sigint() {
    let port = reserve_port();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg(fixture_model_path())
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
        for line in std::io::BufRead::lines(reader) {
            if let Ok(line) = line {
                logs_reader.lock().expect("lock logs").push(line);
            }
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
    assert_success_payload_matches_fixture(&payload);

    wait_for_exit_ok(&mut child, Duration::from_secs(5));
    stderr_thread.join().expect("join stderr reader");
}

#[test]
fn startup_missing_artifact_infer_error_matches_fixture() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("afterburner-http"))
        .arg("does-not-exist.mpk")
        .env("BACKEND", "cpu")
        .env("PORT", reserve_port().to_string())
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
