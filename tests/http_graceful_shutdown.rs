#![cfg(unix)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
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

    wait_for_exit_ok(&mut child, Duration::from_secs(5));
    stderr_thread.join().expect("join stderr reader");
}
