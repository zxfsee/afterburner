use std::net::SocketAddr;

use afterburner::infer::{InferError, load_model, logits_from_model, parse_weights_path_from_args};
use afterburner::observability::json_escape;
use base64::Engine as _;
use burn::prelude::*;
use serde_json::json;
use tiny_http::{Header, Method, Response, Server, StatusCode};

use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

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

    for request in server.incoming_requests() {
        let mut request = request;
        let response = match (request.method(), request.url()) {
            (&Method::Get, "/healthz") => ok_response("ok\n"),
            (&Method::Post, "/infer") => handle_infer(&mut request, &model, &device, backend),
            _ => Response::from_string("not found\n").with_status_code(StatusCode(404)),
        };
        let _ = request.respond(response);
    }

    std::process::exit(2);
}

fn handle_infer<B: Backend>(
    request: &mut tiny_http::Request,
    model: &afterburner::model::Model<B>,
    device: &B::Device,
    backend: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let start = std::time::Instant::now();
    let body = match read_body(request) {
        Ok(body) => body,
        Err(err) => return json_error(StatusCode(400), "invalid_body", &err),
    };

    let content_type = request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Content-Type"))
        .map(|header| header.value.as_str());

    let bytes = match decode_image_bytes(content_type, &body) {
        Ok(bytes) => bytes,
        Err(err) => return json_error(StatusCode(400), "invalid_input", &err),
    };

    let image = match bytes_to_image(&bytes) {
        Ok(image) => image,
        Err(err) => return json_error(StatusCode(400), "invalid_input", &err),
    };

    let logits = logits_from_model(model, device, image);
    let data = logits.to_data();
    let logits: Vec<f32> = data.iter().collect();
    let elapsed_ms = start.elapsed().as_millis();
    eprintln!(r#"{{"event":"infer_done","backend":"{backend}","elapsed_ms":{elapsed_ms}}}"#);

    let payload = json!({ "logits": logits }).to_string();
    Response::from_string(payload)
        .with_header(content_type_json())
        .with_status_code(StatusCode(200))
}

fn read_body(request: &mut tiny_http::Request) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    let mut reader = request.as_reader();
    std::io::Read::read_to_end(&mut reader, &mut body).map_err(|err| err.to_string())?;
    Ok(body)
}

fn decode_image_bytes(content_type: Option<&str>, body: &[u8]) -> Result<Vec<u8>, String> {
    match content_type {
        Some(ct) if ct.starts_with("application/octet-stream") => Ok(body.to_vec()),
        Some(ct) if ct.starts_with("application/json") => decode_json_body(body),
        _ => decode_raw_or_base64(body),
    }
}

fn decode_json_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|err| format!("invalid json: {err}"))?;
    if let Some(base64_value) = value.get("base64").and_then(|v| v.as_str()) {
        decode_base64(base64_value)
    } else if let Some(bytes) = value.get("bytes").and_then(|v| v.as_array()) {
        let mut out = Vec::with_capacity(bytes.len());
        for v in bytes {
            let byte = v
                .as_u64()
                .and_then(|v| u8::try_from(v).ok())
                .ok_or_else(|| "bytes must be 0..255".to_string())?;
            out.push(byte);
        }
        Ok(out)
    } else {
        Err("expected base64 or bytes field".to_string())
    }
}

fn decode_raw_or_base64(body: &[u8]) -> Result<Vec<u8>, String> {
    if body.len() == 28 * 28 {
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
    if bytes.len() != 28 * 28 {
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
