use std::net::SocketAddr;
use std::time::{Duration, Instant};

use afterburner::infer::{InferError, load_model, parse_weights_path_from_args};
use afterburner::observability::json_escape;
use afterburner::preprocess::mnist_image_to_tensor;
use base64::Engine as _;
use burn::prelude::*;
use serde_json::json;
use tiny_http::{Header, Method, Response, Server, StatusCode};

use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

const MNIST_IMAGE_BYTES: usize = 28 * 28;
const LOGIT_CLASSES: usize = 10;
const DEFAULT_HTTP_MAX_REQUEST_BYTES: usize = 64 * 1024;
const DEFAULT_HTTP_MAX_BATCH_SIZE: usize = 16;
const DEFAULT_HTTP_INFER_TIMEOUT_MS: u64 = 1_500;

fn main() {
    let weights_path = parse_weights_path_from_args(std::env::args());
    let addr = http_addr();

    let use_cpu = std::env::var("BACKEND")
        .map(|v| v.eq_ignore_ascii_case("cpu"))
        .unwrap_or(false);

    if use_cpu {
        run_server::<CpuBackend>(weights_path, addr, "cpu");
    } else {
        run_server::<GpuBackend>(weights_path, addr, "wgpu");
    }
}

fn http_addr() -> SocketAddr {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    SocketAddr::from(([0, 0, 0, 0], port))
}

fn run_server<B: Backend>(weights_path: std::path::PathBuf, addr: SocketAddr, backend: &str) -> ! {
    let device = B::Device::default();
    let model = match load_model::<B>(&weights_path, &device) {
        Ok(model) => model,
        Err(err) => emit_error_and_exit(&err),
    };
    let artifact = json_escape(&weights_path.to_string_lossy());
    eprintln!(r#"{{"event":"artifact_load_ok","backend":"{backend}","artifact":"{artifact}"}}"#);
    eprintln!(r#"{{"event":"backend_selected","backend":"{backend}"}}"#);

    let server = Server::http(addr).unwrap_or_else(|err| {
        eprintln!(
            r#"{{"event":"http_error","kind":"bind_failed","detail":"{}"}}"#,
            json_escape(&err.to_string())
        );
        std::process::exit(2);
    });

    let envelope = EnvelopeLimits::from_env();
    eprintln!(
        r#"{{"event":"http_limits","max_request_bytes":{},"max_batch_size":{},"timeout_ms":{}}}"#,
        envelope.max_request_bytes, envelope.max_batch_size, envelope.infer_timeout_ms
    );

    for request in server.incoming_requests() {
        let mut request = request;
        let response = match (request.method(), request.url()) {
            (&Method::Get, "/healthz") => ok_response("ok\n"),
            (&Method::Post, "/infer") => {
                handle_infer(&mut request, &model, &device, backend, envelope)
            }
            _ => Response::from_string("not found\n").with_status_code(StatusCode(404)),
        };
        let _ = request.respond(response);
    }

    std::process::exit(2);
}

#[derive(Clone, Copy, Debug)]
struct EnvelopeLimits {
    max_request_bytes: usize,
    max_batch_size: usize,
    infer_timeout_ms: u64,
}

impl EnvelopeLimits {
    fn from_env() -> Self {
        Self {
            max_request_bytes: parse_env_usize(
                "HTTP_MAX_REQUEST_BYTES",
                DEFAULT_HTTP_MAX_REQUEST_BYTES,
            ),
            max_batch_size: parse_env_usize("HTTP_MAX_BATCH_SIZE", DEFAULT_HTTP_MAX_BATCH_SIZE),
            infer_timeout_ms: parse_env_u64("HTTP_INFER_TIMEOUT_MS", DEFAULT_HTTP_INFER_TIMEOUT_MS),
        }
    }
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn parse_env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

#[derive(Debug)]
struct HttpFailure {
    status: StatusCode,
    kind: &'static str,
    detail: String,
}

impl HttpFailure {
    fn bad_request(kind: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode(400),
            kind,
            detail: detail.into(),
        }
    }

    fn payload_too_large(kind: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode(413),
            kind,
            detail: detail.into(),
        }
    }

    fn unprocessable(kind: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode(422),
            kind,
            detail: detail.into(),
        }
    }

    fn timeout(stage: &'static str, timeout_ms: u64) -> Self {
        Self {
            status: StatusCode(408),
            kind: "infer_timeout",
            detail: format!("timeout exceeded during {stage} (limit: {timeout_ms}ms)"),
        }
    }

    fn internal(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode(500),
            kind: "internal_error",
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy)]
struct Deadline {
    start: Instant,
    timeout_ms: u64,
}

impl Deadline {
    fn new(timeout_ms: u64) -> Self {
        Self {
            start: Instant::now(),
            timeout_ms,
        }
    }

    fn check(&self, stage: &'static str) -> Result<(), HttpFailure> {
        if self.start.elapsed() > Duration::from_millis(self.timeout_ms) {
            return Err(HttpFailure::timeout(stage, self.timeout_ms));
        }
        Ok(())
    }

    fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

fn handle_infer<B: Backend>(
    request: &mut tiny_http::Request,
    model: &afterburner::model::Model<B>,
    device: &B::Device,
    backend: &str,
    envelope: EnvelopeLimits,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let deadline = Deadline::new(envelope.infer_timeout_ms);

    let response = handle_infer_inner(request, model, device, backend, envelope, deadline);
    if let Err(err) = &response {
        eprintln!(
            r#"{{"event":"http_error","kind":"{}","status":{},"detail":"{}"}}"#,
            err.kind,
            err.status.0,
            json_escape(&err.detail)
        );
    }

    match response {
        Ok(payload) => Response::from_string(payload)
            .with_header(content_type_json())
            .with_status_code(StatusCode(200)),
        Err(err) => json_error(err.status, err.kind, &err.detail),
    }
}

fn handle_infer_inner<B: Backend>(
    request: &mut tiny_http::Request,
    model: &afterburner::model::Model<B>,
    device: &B::Device,
    backend: &str,
    envelope: EnvelopeLimits,
    deadline: Deadline,
) -> Result<String, HttpFailure> {
    let body = read_body_limited(request, envelope.max_request_bytes)?;
    deadline.check("request_read")?;

    let content_type = request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Content-Type"))
        .map(|header| header.value.as_str());

    let input_bytes = decode_request_inputs(content_type, &body)
        .map_err(|err| HttpFailure::bad_request("invalid_input", err))?;

    if input_bytes.is_empty() {
        return Err(HttpFailure::bad_request(
            "invalid_input",
            "empty input batch",
        ));
    }
    if input_bytes.len() > envelope.max_batch_size {
        return Err(HttpFailure::payload_too_large(
            "batch_too_large",
            format!(
                "batch size {} exceeds HTTP_MAX_BATCH_SIZE={}",
                input_bytes.len(),
                envelope.max_batch_size
            ),
        ));
    }

    deadline.check("decode")?;

    let images = input_bytes
        .iter()
        .map(|bytes| bytes_to_image(bytes.as_slice()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| HttpFailure::unprocessable("invalid_shape", err))?;

    deadline.check("preprocess")?;

    let logits = logits_from_images(model, device, &images);
    let rows = logits_to_rows(logits, images.len())?;
    deadline.check("inference")?;

    let elapsed_ms = deadline.elapsed_ms();
    eprintln!(
        r#"{{"event":"infer_done","backend":"{backend}","elapsed_ms":{elapsed_ms},"batch_size":{}}}"#,
        images.len()
    );

    if rows.len() == 1 {
        Ok(json!({ "logits": rows[0], "batch_size": 1 }).to_string())
    } else {
        Ok(json!({ "logits": rows, "batch_size": rows.len() }).to_string())
    }
}

fn read_body_limited(
    request: &mut tiny_http::Request,
    max_bytes: usize,
) -> Result<Vec<u8>, HttpFailure> {
    let mut body = Vec::new();
    let mut reader = request.as_reader();
    let mut chunk = [0u8; 8 * 1024];

    loop {
        let read = std::io::Read::read(&mut reader, &mut chunk)
            .map_err(|err| HttpFailure::bad_request("invalid_body", err.to_string()))?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
        if body.len() > max_bytes {
            return Err(HttpFailure::payload_too_large(
                "request_too_large",
                format!("request body exceeds HTTP_MAX_REQUEST_BYTES={max_bytes}"),
            ));
        }
    }

    Ok(body)
}

fn decode_request_inputs(content_type: Option<&str>, body: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    match content_type {
        Some(ct) if ct.starts_with("application/octet-stream") => Ok(vec![body.to_vec()]),
        Some(ct) if ct.starts_with("application/json") => decode_json_inputs(body),
        _ => decode_raw_or_base64(body).map(|single| vec![single]),
    }
}

fn decode_json_inputs(body: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|err| format!("invalid json: {err}"))?;

    if let Some(batch) = value.get("batch").and_then(|v| v.as_array()) {
        return batch
            .iter()
            .map(decode_json_single_input)
            .collect::<Result<Vec<_>, _>>();
    }

    decode_json_single_input(&value).map(|single| vec![single])
}

fn decode_json_single_input(value: &serde_json::Value) -> Result<Vec<u8>, String> {
    if let Some(base64_value) = value.get("base64").and_then(|v| v.as_str()) {
        return decode_base64(base64_value);
    }

    if let Some(bytes) = value.get("bytes").and_then(|v| v.as_array()) {
        let mut out = Vec::with_capacity(bytes.len());
        for v in bytes {
            let byte = v
                .as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(|| "bytes must be 0..255".to_string())?;
            out.push(byte);
        }
        return Ok(out);
    }

    Err("expected base64 or bytes field".to_string())
}

fn decode_raw_or_base64(body: &[u8]) -> Result<Vec<u8>, String> {
    if body.len() == MNIST_IMAGE_BYTES {
        return Ok(body.to_vec());
    }
    let text = std::str::from_utf8(body).map_err(|_| "invalid utf-8 body".to_string())?;
    decode_base64(text.trim())
}

fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|err| format!("invalid base64: {err}"))
}

fn bytes_to_image(bytes: &[u8]) -> Result<[[f32; 28]; 28], String> {
    if bytes.len() != MNIST_IMAGE_BYTES {
        return Err("expected 784 bytes".to_string());
    }
    let mut image = [[0.0f32; 28]; 28];
    for y in 0..28 {
        for x in 0..28 {
            image[y][x] = bytes[y * 28 + x] as f32;
        }
    }
    Ok(image)
}

fn logits_from_images<B: Backend>(
    model: &afterburner::model::Model<B>,
    device: &B::Device,
    images: &[[[f32; 28]; 28]],
) -> Tensor<B, 2> {
    let tensors = images
        .iter()
        .map(|image| mnist_image_to_tensor::<B>(*image, device))
        .collect::<Vec<_>>();
    let input = Tensor::stack(tensors, 0);
    model.forward(input)
}

fn logits_to_rows<B: Backend>(
    logits: Tensor<B, 2>,
    batch_size: usize,
) -> Result<Vec<Vec<f32>>, HttpFailure> {
    let values: Vec<f32> = logits.to_data().iter().collect();
    let expected_values = batch_size * LOGIT_CLASSES;
    if values.len() != expected_values {
        return Err(HttpFailure::internal(format!(
            "unexpected logits shape: expected {} values, got {}",
            expected_values,
            values.len()
        )));
    }

    Ok(values
        .chunks_exact(LOGIT_CLASSES)
        .map(|chunk| chunk.to_vec())
        .collect())
}

fn ok_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_status_code(StatusCode(200))
}

fn json_error(status: StatusCode, kind: &str, detail: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let payload = json!({ "error": { "kind": kind, "detail": detail } }).to_string();
    Response::from_string(payload)
        .with_header(content_type_json())
        .with_status_code(status)
}

fn content_type_json() -> Header {
    Header::from_bytes("Content-Type", "application/json").expect("valid header")
}

fn emit_error_and_exit(err: &InferError) -> ! {
    match err {
        InferError::ArtifactMissing { path } => {
            let artifact = json_escape(&path.to_string_lossy());
            eprintln!(
                r#"{{"event":"infer_error","kind":"artifact_missing","artifact":"{artifact}"}}"#
            );
        }
        InferError::ManifestMissing { path } => {
            let manifest = json_escape(&path.to_string_lossy());
            eprintln!(
                r#"{{"event":"infer_error","kind":"manifest_missing","manifest":"{manifest}"}}"#
            );
        }
        InferError::ManifestInvalid(manifest_err) => match manifest_err {
            afterburner::manifest::ManifestError::Io(err) => {
                let detail = json_escape(&err.to_string());
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_io_error","detail":"{detail}"}}"#
                );
            }
            afterburner::manifest::ManifestError::TomlParse(err) => {
                let detail = json_escape(&err.to_string());
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_parse_error","detail":"{detail}"}}"#
                );
            }
            afterburner::manifest::ManifestError::MissingField(field) => {
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_missing_field","field":"{field}"}}"#
                );
            }
            afterburner::manifest::ManifestError::InvalidField(field, detail) => {
                let detail = json_escape(detail);
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_invalid_field","field":"{field}","detail":"{detail}"}}"#
                );
            }
            afterburner::manifest::ManifestError::Mismatch {
                field,
                expected,
                actual,
            } => {
                let expected = json_escape(expected);
                let actual = json_escape(actual);
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_mismatch","field":"{field}","expected":"{expected}","actual":"{actual}"}}"#
                );
            }
        },
        InferError::ArtifactLoadFailed { detail, path } => {
            let detail = json_escape(detail);
            let artifact = json_escape(&path.to_string_lossy());
            eprintln!(
                r#"{{"event":"infer_error","kind":"artifact_load_failed","artifact":"{artifact}","detail":"{detail}"}}"#
            );
        }
    }
    std::process::exit(2);
}

#[cfg(test)]
mod tests {
    use super::{Deadline, decode_json_inputs, decode_request_inputs};

    #[test]
    fn decode_single_json_base64() {
        let body = br#"{"base64":"AA=="}"#;
        let decoded = decode_json_inputs(body).expect("valid decode");
        assert_eq!(decoded, vec![vec![0u8]]);
    }

    #[test]
    fn decode_json_batch_mixed_inputs() {
        let body = br#"{"batch":[{"bytes":[1,2,3]},{"base64":"BAU="}]}"#;
        let decoded = decode_json_inputs(body).expect("valid decode");
        assert_eq!(decoded, vec![vec![1u8, 2, 3], vec![4u8, 5]]);
    }

    #[test]
    fn decode_json_batch_requires_shape() {
        let body = br#"{"batch":[{"not_bytes":true}]}"#;
        let err = decode_json_inputs(body).expect_err("invalid decode must fail");
        assert!(err.contains("expected base64 or bytes field"));
    }

    #[test]
    fn decode_request_defaults_to_single_for_octet_stream() {
        let body = vec![7u8; 28 * 28];
        let decoded = decode_request_inputs(Some("application/octet-stream"), &body)
            .expect("octet stream should decode");
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].len(), 28 * 28);
    }

    #[test]
    fn timeout_check_fails_after_elapsed() {
        let deadline = Deadline::new(1);
        std::thread::sleep(std::time::Duration::from_millis(3));
        let err = deadline.check("decode").expect_err("must timeout");
        assert_eq!(err.kind, "infer_timeout");
    }
}
