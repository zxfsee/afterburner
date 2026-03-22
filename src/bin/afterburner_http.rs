use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use afterburner::infer::{
    InferError, load_model, manifest_path_for_weights, parse_weights_path_from_args,
    validate_artifacts,
};
use afterburner::manifest::{ArtifactManifest, ManifestPrecision};
use afterburner::observability::{
    TRACEPARENT_HEADER, TraceContext, attach_trace_fields, emit_event,
    trace_context_from_optional_traceparent,
};
use afterburner::preprocess::mnist_image_to_tensor;
use base64::Engine as _;
use burn::prelude::*;
use serde_json::{Value, json};
use tiny_http::{Header, Method, Response, Server, StatusCode};

use burn::{
    backend::ndarray::NdArray,
    backend::wgpu::{Metal, Wgpu},
};

type GpuBackend = Wgpu<f32, i32>;
type MetalBackend = Metal<f32, i32>;
type CpuBackend = NdArray<f32>;

const MNIST_IMAGE_BYTES: usize = 28 * 28;
const LOGIT_CLASSES: usize = 10;
const DEFAULT_HTTP_MAX_REQUEST_BYTES: usize = 64 * 1024;
const DEFAULT_HTTP_MAX_BATCH_SIZE: usize = 16;
const DEFAULT_HTTP_INFER_TIMEOUT_MS: u64 = 1_500;
const DEFAULT_HTTP_MAX_CONCURRENCY: usize = 4;
const EXTRA_HTTP_ACCEPT_WORKERS: usize = 1;
const HTTP_RECV_TIMEOUT_MS: u64 = 200;
const HTTP_ROLLOUT_BUDGET_METADATA_ENV: &str = "HTTP_ROLLOUT_BUDGET_METADATA_JSON";
const ROLLOUT_BUDGET_SCHEMA_JSON: &str =
    include_str!("../../fixtures/serving_rollout_budget_metadata.schema.json");
const ROLLOUT_LATENCY_HEADER: &str = "X-Rollout-Observed-Latency-P99-Ms";
const ROLLOUT_ERROR_HEADER: &str = "X-Rollout-Observed-Error-Ratio";

fn main() {
    let weights_path = parse_weights_path_from_args(std::env::args());
    let addr = http_addr();

    let backend = std::env::var("BACKEND")
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_else(|_| "wgpu".to_string());

    match backend.as_str() {
        "cpu" => run_server::<CpuBackend>(weights_path, addr, "cpu"),
        "metal" => run_server::<MetalBackend>(weights_path, addr, "metal"),
        _ => run_server::<GpuBackend>(weights_path, addr, "wgpu"),
    }
}

fn http_addr() -> SocketAddr {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    SocketAddr::from(([0, 0, 0, 0], port))
}

fn run_server<B>(weights_path: std::path::PathBuf, addr: SocketAddr, backend: &'static str) -> !
where
    B: Backend + Send + 'static,
    B::Device: Send + 'static,
{
    let manifest = load_artifact_manifest(&weights_path).unwrap_or_else(|err| {
        emit_error_and_exit(&err);
    });
    let artifact_version = manifest.artifact_version.clone();
    let precision = manifest.precision.clone();
    ensure_model_loadable::<B>(&weights_path, backend, &precision);
    emit_event(
        "info",
        "http_adapter",
        "backend_selected",
        json!({"backend": backend}),
    );

    let server = Arc::new(Server::http(addr).unwrap_or_else(|err| {
        emit_event(
            "error",
            "http_adapter",
            "http_error",
            json!({"kind": "bind_failed", "detail": err.to_string()}),
        );
        std::process::exit(2);
    }));

    let envelope = EnvelopeLimits::from_env();
    let infer_limiter = Arc::new(InferConcurrencyLimiter::new(envelope.max_concurrency));
    let worker_count = envelope
        .max_concurrency
        .saturating_add(EXTRA_HTTP_ACCEPT_WORKERS);
    let shutdown = Arc::new(AtomicBool::new(false));
    let request_ids = Arc::new(RequestIdGenerator::new());
    emit_event(
        "info",
        "http_adapter",
        "http_limits",
        json!({
            "max_request_bytes": envelope.max_request_bytes,
            "max_batch_size": envelope.max_batch_size,
            "timeout_ms": envelope.infer_timeout_ms,
            "max_concurrency": envelope.max_concurrency,
            "rollout_budget_enforced": envelope.rollout_budget.is_some(),
            "worker_count": worker_count
        }),
    );
    if let Some(rollout_budget) = envelope.rollout_budget.as_ref() {
        emit_event(
            "info",
            "http_adapter",
            "rollout_budget_loaded",
            json!({
                "service": rollout_budget.service,
                "window": rollout_budget.window,
                "latency_budget_ms_p99": rollout_budget.latency_budget_ms_p99,
                "error_budget_ratio": rollout_budget.error_budget_ratio
            }),
        );
    }
    emit_event(
        "info",
        "http_adapter",
        "http_start",
        json!({
            "backend": backend,
            "addr": addr.to_string(),
            "max_concurrency": envelope.max_concurrency,
            "rollout_budget_enforced": envelope.rollout_budget.is_some(),
            "worker_count": worker_count
        }),
    );

    install_signal_handler(Arc::clone(&shutdown), Arc::clone(&server), worker_count);

    let mut worker_handles = Vec::with_capacity(worker_count);
    for worker_idx in 0..worker_count {
        let server = Arc::clone(&server);
        let shutdown = Arc::clone(&shutdown);
        let request_ids = Arc::clone(&request_ids);
        let infer_limiter = Arc::clone(&infer_limiter);
        let worker_backend = backend;
        let worker_weights = weights_path.clone();
        let worker_artifact_version = artifact_version.clone();
        let worker_precision = precision.clone();
        let worker_envelope = envelope.clone();
        let worker_handle = thread::spawn(move || {
            run_worker_loop::<B>(
                worker_idx,
                server,
                shutdown,
                request_ids,
                worker_backend,
                worker_envelope,
                infer_limiter,
                worker_weights,
                worker_artifact_version,
                worker_precision,
            );
        });
        worker_handles.push(worker_handle);
    }

    for worker_handle in worker_handles {
        let _ = worker_handle.join();
    }

    emit_event(
        "info",
        "http_adapter",
        "http_shutdown",
        json!({"status":"ok"}),
    );
    std::process::exit(0);
}

fn ensure_model_loadable<B: Backend>(
    weights_path: &std::path::Path,
    backend: &str,
    precision: &ManifestPrecision,
) {
    let device = B::Device::default();
    match load_model::<B>(&weights_path.to_path_buf(), &device) {
        Ok(_) => {
            emit_event(
                "info",
                "http_adapter",
                "artifact_load_ok",
                json!({
                    "backend": backend,
                    "artifact": weights_path.to_string_lossy().to_string(),
                    "precision": precision_json(precision)
                }),
            );
        }
        Err(err) => emit_error_and_exit(&err),
    }
}

fn load_artifact_manifest(weights_path: &std::path::Path) -> Result<ArtifactManifest, InferError> {
    validate_artifacts(weights_path)?;
    let manifest_path = manifest_path_for_weights(weights_path);
    ArtifactManifest::load_from_path(&manifest_path).map_err(InferError::ManifestInvalid)
}

#[derive(Clone, Debug)]
struct EnvelopeLimits {
    max_request_bytes: usize,
    max_batch_size: usize,
    infer_timeout_ms: u64,
    max_concurrency: usize,
    rollout_budget: Option<RolloutBudgetMetadata>,
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
            max_concurrency: parse_env_usize("HTTP_MAX_CONCURRENCY", DEFAULT_HTTP_MAX_CONCURRENCY),
            rollout_budget: load_rollout_budget_metadata_from_env(),
        }
    }
}

#[derive(Clone, Debug)]
struct RolloutBudgetMetadata {
    service: String,
    latency_budget_ms_p99: u64,
    error_budget_ratio: f64,
    window: String,
}

#[derive(Debug)]
struct RolloutBudgetSchema {
    required_fields: Vec<String>,
    allowed_fields: Vec<String>,
    schema_version: String,
    service_min_length: usize,
    latency_minimum: u64,
    error_minimum: f64,
    error_maximum: f64,
    window_min_length: usize,
}

#[derive(Debug, Default)]
struct RequestIdGenerator {
    next: AtomicU64,
}

impl RequestIdGenerator {
    fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }
}

fn install_signal_handler(shutdown: Arc<AtomicBool>, server: Arc<Server>, max_concurrency: usize) {
    ctrlc::set_handler(move || {
        shutdown.store(true, Ordering::SeqCst);
        for _ in 0..max_concurrency {
            server.unblock();
        }
    })
    .unwrap_or_else(|err| {
        emit_event(
            "error",
            "http_adapter",
            "http_error",
            json!({"kind":"signal_handler_install_failed","detail": err.to_string()}),
        );
        std::process::exit(2);
    });
}

fn run_worker_loop<B>(
    worker_idx: usize,
    server: Arc<Server>,
    shutdown: Arc<AtomicBool>,
    request_ids: Arc<RequestIdGenerator>,
    backend: &str,
    envelope: EnvelopeLimits,
    infer_limiter: Arc<InferConcurrencyLimiter>,
    weights_path: std::path::PathBuf,
    artifact_version: String,
    precision: ManifestPrecision,
) where
    B: Backend + Send + 'static,
    B::Device: Send + 'static,
{
    let recv_timeout = Duration::from_millis(HTTP_RECV_TIMEOUT_MS);
    let device = B::Device::default();
    let model = match load_model::<B>(&weights_path, &device) {
        Ok(model) => model,
        Err(err) => emit_error_and_exit(&err),
    };
    emit_event(
        "info",
        "http_adapter",
        "http_worker_start",
        json!({"worker": worker_idx}),
    );

    while !shutdown.load(Ordering::Relaxed) {
        let mut request = match server.recv_timeout(recv_timeout) {
            Ok(Some(request)) => request,
            Ok(None) => continue,
            Err(err) => {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                emit_event(
                    "error",
                    "http_adapter",
                    "http_error",
                    json!({"kind":"recv_failed","detail": err.to_string()}),
                );
                continue;
            }
        };

        let request_id = request_ids.next_id();
        let response = route_request(
            request_id,
            &mut request,
            &model,
            &device,
            backend,
            artifact_version.as_str(),
            &precision,
            &envelope,
            &infer_limiter,
            worker_idx,
        );
        let _ = request.respond(response);
    }

    emit_event(
        "info",
        "http_adapter",
        "http_worker_stop",
        json!({"worker": worker_idx}),
    );
}

fn route_request<B: Backend>(
    request_id: u64,
    request: &mut tiny_http::Request,
    model: &afterburner::model::Model<B>,
    device: &B::Device,
    backend: &str,
    artifact_version: &str,
    precision: &ManifestPrecision,
    envelope: &EnvelopeLimits,
    infer_limiter: &InferConcurrencyLimiter,
    worker_idx: usize,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let trace = trace_context_from_optional_traceparent(
        header_value(request, TRACEPARENT_HEADER),
        "http_request",
    );
    let mut request_fields = serde_json::Map::from_iter([
        (
            "method".to_string(),
            Value::from(request.method().as_str().to_string()),
        ),
        ("path".to_string(), Value::from(request.url().to_string())),
        (
            "request_id".to_string(),
            Value::from(request_id.to_string()),
        ),
        ("worker".to_string(), Value::from(worker_idx)),
    ]);
    attach_trace_fields(&mut request_fields, &trace);
    emit_event(
        "info",
        "http_adapter",
        "http_request",
        Value::Object(request_fields),
    );

    let response = match (request.method(), request.url()) {
        (&Method::Get, "/healthz") => ok_response("ok\n"),
        (&Method::Post, "/infer") => handle_infer(
            request_id,
            &trace,
            request,
            model,
            device,
            backend,
            artifact_version,
            precision,
            envelope,
            infer_limiter,
        ),
        _ => Response::from_string("not found\n").with_status_code(StatusCode(404)),
    };
    with_request_context(response, request_id, &trace)
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| parse_positive_usize(&value))
        .unwrap_or(default)
}

fn parse_env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| parse_positive_u64(&value))
        .unwrap_or(default)
}

fn parse_positive_usize(value: &str) -> Option<usize> {
    value.parse::<usize>().ok().filter(|value| *value > 0)
}

fn parse_positive_u64(value: &str) -> Option<u64> {
    value.parse::<u64>().ok().filter(|value| *value > 0)
}

fn load_rollout_budget_metadata_from_env() -> Option<RolloutBudgetMetadata> {
    let raw = match std::env::var(HTTP_ROLLOUT_BUDGET_METADATA_ENV) {
        Ok(raw) => raw,
        Err(_) => return None,
    };

    match parse_rollout_budget_metadata(raw.as_str()) {
        Ok(metadata) => Some(metadata),
        Err(detail) => {
            emit_event(
                "error",
                "http_adapter",
                "http_error",
                json!({
                    "kind":"invalid_rollout_budget_metadata",
                    "detail": detail
                }),
            );
            None
        }
    }
}

fn parse_rollout_budget_metadata(raw: &str) -> Result<RolloutBudgetMetadata, String> {
    let schema = parse_rollout_budget_schema()?;
    let value: Value = serde_json::from_str(raw).map_err(|err| format!("invalid json: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "rollout metadata must be an object".to_string())?;
    for required_field in &schema.required_fields {
        if !object.contains_key(required_field) {
            return Err(format!(
                "missing required field `{required_field}` in rollout metadata"
            ));
        }
    }
    for key in object.keys() {
        if !schema.allowed_fields.iter().any(|expected| expected == key) {
            return Err(format!("unknown field `{key}` in rollout metadata"));
        }
    }

    let schema_version = object
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| "schema_version must be a string".to_string())?;
    if schema_version != schema.schema_version.as_str() {
        return Err(format!(
            "schema_version must be \"{}\"",
            schema.schema_version
        ));
    }

    let service = object
        .get("service")
        .and_then(Value::as_str)
        .ok_or_else(|| "service must be a string".to_string())?;
    if service.chars().count() < schema.service_min_length {
        return Err(format!(
            "service must have minLength >= {}",
            schema.service_min_length
        ));
    }

    let latency_budget_ms_p99 = object
        .get("latency_budget_ms_p99")
        .and_then(Value::as_u64)
        .ok_or_else(|| "latency_budget_ms_p99 must be an integer".to_string())?;
    if latency_budget_ms_p99 < schema.latency_minimum {
        return Err(format!(
            "latency_budget_ms_p99 must be >= {}",
            schema.latency_minimum
        ));
    }

    let error_budget_ratio = object
        .get("error_budget_ratio")
        .and_then(Value::as_f64)
        .ok_or_else(|| "error_budget_ratio must be a number in [0, 1]".to_string())?;
    if !error_budget_ratio.is_finite()
        || error_budget_ratio < schema.error_minimum
        || error_budget_ratio > schema.error_maximum
    {
        return Err("error_budget_ratio must be a number in [0, 1]".to_string());
    }

    let window = object
        .get("window")
        .and_then(Value::as_str)
        .ok_or_else(|| "window must be a string".to_string())?;
    if window.chars().count() < schema.window_min_length {
        return Err(format!(
            "window must have minLength >= {}",
            schema.window_min_length
        ));
    }

    Ok(RolloutBudgetMetadata {
        service: service.to_string(),
        latency_budget_ms_p99,
        error_budget_ratio,
        window: window.to_string(),
    })
}

fn parse_rollout_budget_schema() -> Result<RolloutBudgetSchema, String> {
    let value: Value = serde_json::from_str(ROLLOUT_BUDGET_SCHEMA_JSON)
        .map_err(|err| format!("invalid rollout budget schema fixture json: {err}"))?;
    let root = value
        .as_object()
        .ok_or_else(|| "rollout budget schema fixture must be a json object".to_string())?;

    let root_type = root
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "rollout budget schema fixture missing `type`".to_string())?;
    if root_type != "object" {
        return Err("rollout budget schema fixture `type` must be `object`".to_string());
    }

    let additional_properties = root
        .get("additionalProperties")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            "rollout budget schema fixture missing `additionalProperties`".to_string()
        })?;
    if additional_properties {
        return Err(
            "rollout budget schema fixture must set `additionalProperties` to false".to_string(),
        );
    }

    let required_fields =
        root.get("required")
            .and_then(Value::as_array)
            .ok_or_else(|| "rollout budget schema fixture missing `required`".to_string())?
            .iter()
            .map(|value| {
                value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                    "rollout budget schema `required` items must be strings".to_string()
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

    let properties = root
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "rollout budget schema fixture missing `properties`".to_string())?;
    let allowed_fields = properties.keys().cloned().collect::<Vec<_>>();

    let schema_version = schema_property_const(properties, "schema_version")?;
    let service_min_length = schema_property_string_min_length(properties, "service")?;
    let latency_minimum = schema_property_integer_minimum(properties, "latency_budget_ms_p99")?;
    let (error_minimum, error_maximum) =
        schema_property_number_range(properties, "error_budget_ratio")?;
    let window_min_length = schema_property_string_min_length(properties, "window")?;

    Ok(RolloutBudgetSchema {
        required_fields,
        allowed_fields,
        schema_version,
        service_min_length,
        latency_minimum,
        error_minimum,
        error_maximum,
        window_min_length,
    })
}

fn schema_property_const(
    properties: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, String> {
    let property = schema_property(properties, field)?;
    property
        .get("const")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("rollout budget schema property `{field}` missing string `const`"))
}

fn schema_property_string_min_length(
    properties: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<usize, String> {
    let property = schema_property(properties, field)?;
    let field_type = property
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("rollout budget schema property `{field}` missing `type`"))?;
    if field_type != "string" {
        return Err(format!(
            "rollout budget schema property `{field}` must use `type: string`"
        ));
    }

    let min_length = property
        .get("minLength")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("rollout budget schema property `{field}` missing `minLength`"))?;
    usize::try_from(min_length).map_err(|_| {
        format!("rollout budget schema property `{field}` minLength {min_length} exceeds usize")
    })
}

fn schema_property_integer_minimum(
    properties: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<u64, String> {
    let property = schema_property(properties, field)?;
    let field_type = property
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("rollout budget schema property `{field}` missing `type`"))?;
    if field_type != "integer" {
        return Err(format!(
            "rollout budget schema property `{field}` must use `type: integer`"
        ));
    }

    property
        .get("minimum")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            format!("rollout budget schema property `{field}` missing integer `minimum`")
        })
}

fn schema_property_number_range(
    properties: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(f64, f64), String> {
    let property = schema_property(properties, field)?;
    let field_type = property
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("rollout budget schema property `{field}` missing `type`"))?;
    if field_type != "number" {
        return Err(format!(
            "rollout budget schema property `{field}` must use `type: number`"
        ));
    }

    let minimum = property
        .get("minimum")
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            format!("rollout budget schema property `{field}` missing number `minimum`")
        })?;
    let maximum = property
        .get("maximum")
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            format!("rollout budget schema property `{field}` missing number `maximum`")
        })?;
    Ok((minimum, maximum))
}

fn schema_property<'a>(
    properties: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    properties
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("rollout budget schema fixture missing `{field}` property"))
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

    fn service_unavailable(kind: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode(503),
            kind,
            detail: detail.into(),
        }
    }
}

#[derive(Debug)]
struct InferConcurrencyLimiter {
    in_flight: AtomicUsize,
    max_concurrency: usize,
}

impl InferConcurrencyLimiter {
    fn new(max_concurrency: usize) -> Self {
        Self {
            in_flight: AtomicUsize::new(0),
            max_concurrency,
        }
    }

    fn try_acquire(&self) -> Option<InferConcurrencyGuard<'_>> {
        loop {
            let current = self.in_flight.load(Ordering::Relaxed);
            if current >= self.max_concurrency {
                return None;
            }
            let next = current + 1;
            if self
                .in_flight
                .compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return Some(InferConcurrencyGuard { limiter: self });
            }
        }
    }
}

struct InferConcurrencyGuard<'a> {
    limiter: &'a InferConcurrencyLimiter,
}

impl Drop for InferConcurrencyGuard<'_> {
    fn drop(&mut self) {
        self.limiter.in_flight.fetch_sub(1, Ordering::AcqRel);
    }
}

fn infer_overload_failure(max_concurrency: usize) -> HttpFailure {
    HttpFailure::service_unavailable(
        "server_overloaded",
        format!("HTTP_MAX_CONCURRENCY={max_concurrency} saturated"),
    )
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
    request_id: u64,
    trace: &TraceContext,
    request: &mut tiny_http::Request,
    model: &afterburner::model::Model<B>,
    device: &B::Device,
    backend: &str,
    artifact_version: &str,
    precision: &ManifestPrecision,
    envelope: &EnvelopeLimits,
    infer_limiter: &InferConcurrencyLimiter,
) -> Response<std::io::Cursor<Vec<u8>>> {
    if let Err(err) = evaluate_rollout_budget_admission(request_id, trace, request, envelope) {
        return json_error(err.status, err.kind, &err.detail);
    }

    let _permit = match infer_limiter.try_acquire() {
        Some(permit) => permit,
        None => {
            let err = infer_overload_failure(envelope.max_concurrency);
            return json_error(err.status, err.kind, &err.detail);
        }
    };

    let deadline = Deadline::new(envelope.infer_timeout_ms);

    let response = handle_infer_inner(
        request,
        model,
        device,
        backend,
        artifact_version,
        precision,
        envelope,
        deadline,
        request_id,
        trace,
    );
    if let Err(err) = &response {
        let mut error_fields = serde_json::Map::from_iter([
            (
                "request_id".to_string(),
                Value::from(request_id.to_string()),
            ),
            ("kind".to_string(), Value::from(err.kind)),
            ("status".to_string(), Value::from(err.status.0)),
            ("detail".to_string(), Value::from(err.detail.clone())),
        ]);
        attach_trace_fields(&mut error_fields, trace);
        emit_event(
            "error",
            "http_adapter",
            "http_error",
            Value::Object(error_fields),
        );
    }

    match response {
        Ok(payload) => Response::from_string(payload)
            .with_header(content_type_json())
            .with_status_code(StatusCode(200)),
        Err(err) => json_error(err.status, err.kind, &err.detail),
    }
}

fn evaluate_rollout_budget_admission(
    request_id: u64,
    trace: &TraceContext,
    request: &tiny_http::Request,
    envelope: &EnvelopeLimits,
) -> Result<(), HttpFailure> {
    let rollout_budget = match envelope.rollout_budget.as_ref() {
        Some(metadata) => metadata,
        None => return Ok(()),
    };

    let observed_latency_ms_p99 =
        parse_required_rollout_u64_header(request, ROLLOUT_LATENCY_HEADER)?;
    let observed_error_ratio = parse_required_rollout_f64_header(request, ROLLOUT_ERROR_HEADER)?;
    if !(0.0..=1.0).contains(&observed_error_ratio) {
        return Err(HttpFailure::bad_request(
            "invalid_rollout_observation",
            format!("{ROLLOUT_ERROR_HEADER} must be in [0, 1]"),
        ));
    }

    let admitted = observed_latency_ms_p99 <= rollout_budget.latency_budget_ms_p99
        && observed_error_ratio <= rollout_budget.error_budget_ratio;
    let mut fields = serde_json::Map::from_iter([
        (
            "request_id".to_string(),
            Value::from(request_id.to_string()),
        ),
        (
            "service".to_string(),
            Value::from(rollout_budget.service.clone()),
        ),
        (
            "window".to_string(),
            Value::from(rollout_budget.window.clone()),
        ),
        ("admitted".to_string(), Value::from(admitted)),
        (
            "observed_latency_ms_p99".to_string(),
            Value::from(observed_latency_ms_p99),
        ),
        (
            "latency_budget_ms_p99".to_string(),
            Value::from(rollout_budget.latency_budget_ms_p99),
        ),
        (
            "observed_error_ratio".to_string(),
            Value::from(observed_error_ratio),
        ),
        (
            "error_budget_ratio".to_string(),
            Value::from(rollout_budget.error_budget_ratio),
        ),
    ]);
    attach_trace_fields(&mut fields, trace);
    emit_event(
        "info",
        "http_adapter",
        "rollout_budget_admission",
        Value::Object(fields),
    );
    if admitted {
        return Ok(());
    }

    Err(HttpFailure::service_unavailable(
        "rollout_budget_exceeded",
        format!(
            "rollout budget exceeded: latency_p99_ms={} (budget {}), error_ratio={} (budget {})",
            observed_latency_ms_p99,
            rollout_budget.latency_budget_ms_p99,
            observed_error_ratio,
            rollout_budget.error_budget_ratio
        ),
    ))
}

fn parse_required_rollout_u64_header(
    request: &tiny_http::Request,
    header_name: &str,
) -> Result<u64, HttpFailure> {
    let raw = header_value(request, header_name).ok_or_else(|| {
        HttpFailure::bad_request(
            "invalid_rollout_observation",
            format!("missing {header_name} header"),
        )
    })?;

    raw.parse::<u64>().map_err(|_| {
        HttpFailure::bad_request(
            "invalid_rollout_observation",
            format!("{header_name} must be an unsigned integer"),
        )
    })
}

fn parse_required_rollout_f64_header(
    request: &tiny_http::Request,
    header_name: &str,
) -> Result<f64, HttpFailure> {
    let raw = header_value(request, header_name).ok_or_else(|| {
        HttpFailure::bad_request(
            "invalid_rollout_observation",
            format!("missing {header_name} header"),
        )
    })?;

    let parsed = raw.parse::<f64>().map_err(|_| {
        HttpFailure::bad_request(
            "invalid_rollout_observation",
            format!("{header_name} must be a number"),
        )
    })?;
    if !parsed.is_finite() {
        return Err(HttpFailure::bad_request(
            "invalid_rollout_observation",
            format!("{header_name} must be finite"),
        ));
    }
    Ok(parsed)
}

fn header_value<'a>(request: &'a tiny_http::Request, header_name: &str) -> Option<&'a str> {
    request
        .headers()
        .iter()
        .find(|header| {
            header
                .field
                .as_str()
                .as_str()
                .eq_ignore_ascii_case(header_name)
        })
        .map(|header| header.value.as_str())
}

fn handle_infer_inner<B: Backend>(
    request: &mut tiny_http::Request,
    model: &afterburner::model::Model<B>,
    device: &B::Device,
    backend: &str,
    artifact_version: &str,
    precision: &ManifestPrecision,
    envelope: &EnvelopeLimits,
    deadline: Deadline,
    request_id: u64,
    trace: &TraceContext,
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

    let duration_ms = u64::try_from(deadline.elapsed_ms()).unwrap_or(u64::MAX);
    let mut fields = serde_json::Map::from_iter([
        (
            "request_id".to_string(),
            Value::from(request_id.to_string()),
        ),
        ("backend".to_string(), Value::from(backend.to_string())),
        (
            "artifact_version".to_string(),
            Value::from(artifact_version.to_string()),
        ),
        ("precision".to_string(), precision_json(precision)),
        ("duration_ms".to_string(), Value::from(duration_ms)),
        ("batch_size".to_string(), Value::from(images.len())),
    ]);
    attach_trace_fields(&mut fields, trace);
    emit_event("info", "http_adapter", "infer_done", Value::Object(fields));

    Ok(infer_success_payload(&rows))
}

fn precision_json(precision: &ManifestPrecision) -> Value {
    json!({
        "weights_dtype": precision.weights_dtype,
        "activation_dtype": precision.activation_dtype,
        "quantization": precision.quantization,
    })
}

fn infer_success_payload(rows: &[Vec<f32>]) -> String {
    json!({ "logits": rows, "batch_size": rows.len() }).to_string()
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
    let payload = error_payload(kind, detail);
    Response::from_string(payload)
        .with_header(content_type_json())
        .with_status_code(status)
}

fn error_payload(kind: &str, detail: &str) -> String {
    json!({ "error": { "kind": kind, "detail": detail } }).to_string()
}

fn content_type_json() -> Header {
    Header::from_bytes("Content-Type", "application/json").expect("valid header")
}

fn with_request_context(
    response: Response<std::io::Cursor<Vec<u8>>>,
    request_id: u64,
    trace: &TraceContext,
) -> Response<std::io::Cursor<Vec<u8>>> {
    response
        .with_header(request_id_header(request_id))
        .with_header(traceparent_header(trace.traceparent.as_str()))
}

fn request_id_header(request_id: u64) -> Header {
    Header::from_bytes("X-Request-Id", request_id.to_string()).expect("valid header")
}

fn traceparent_header(traceparent: &str) -> Header {
    Header::from_bytes("Traceparent", traceparent).expect("valid traceparent header")
}

fn emit_error_and_exit(err: &InferError) -> ! {
    match err {
        InferError::ArtifactMissing { path } => {
            emit_event(
                "error",
                "http_adapter",
                "infer_error",
                json!({"kind":"artifact_not_found","artifact": path.to_string_lossy().to_string()}),
            );
        }
        InferError::ManifestMissing { path } => {
            emit_event(
                "error",
                "http_adapter",
                "infer_error",
                json!({"kind":"manifest_missing","manifest": path.to_string_lossy().to_string()}),
            );
        }
        InferError::ManifestInvalid(manifest_err) => match manifest_err {
            afterburner::manifest::ManifestError::Io(err) => {
                emit_event(
                    "error",
                    "http_adapter",
                    "infer_error",
                    json!({"kind":"manifest_io_error","detail": err.to_string()}),
                );
            }
            afterburner::manifest::ManifestError::TomlParse(err) => {
                emit_event(
                    "error",
                    "http_adapter",
                    "infer_error",
                    json!({"kind":"manifest_parse_error","detail": err.to_string()}),
                );
            }
            afterburner::manifest::ManifestError::MissingField(field) => {
                emit_event(
                    "error",
                    "http_adapter",
                    "infer_error",
                    json!({"kind":"manifest_missing_field","field": field}),
                );
            }
            afterburner::manifest::ManifestError::InvalidField(field, detail) => {
                emit_event(
                    "error",
                    "http_adapter",
                    "infer_error",
                    json!({"kind":"manifest_invalid_field","field": field, "detail": detail}),
                );
            }
            afterburner::manifest::ManifestError::Mismatch {
                field,
                expected,
                actual,
            } => {
                emit_event(
                    "error",
                    "http_adapter",
                    "infer_error",
                    json!({"kind":"manifest_mismatch","field": field, "expected": expected, "actual": actual}),
                );
            }
        },
        InferError::ArtifactLoadFailed { detail, path } => {
            emit_event(
                "error",
                "http_adapter",
                "infer_error",
                json!({"kind":"artifact_load_failed","artifact": path.to_string_lossy().to_string(), "detail": detail}),
            );
        }
    }
    std::process::exit(2);
}

#[cfg(test)]
mod tests {
    use super::{
        Deadline, InferConcurrencyLimiter, RequestIdGenerator, decode_json_inputs,
        decode_request_inputs, error_payload, infer_overload_failure, infer_success_payload,
        parse_positive_u64, parse_positive_usize, parse_rollout_budget_metadata, request_id_header,
    };
    use tiny_http::StatusCode;

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

    #[test]
    fn parse_positive_numbers_rejects_zero_and_invalid() {
        assert_eq!(parse_positive_usize("16"), Some(16));
        assert_eq!(parse_positive_usize("0"), None);
        assert_eq!(parse_positive_usize("bad"), None);
        assert_eq!(parse_positive_u64("1500"), Some(1500));
        assert_eq!(parse_positive_u64("0"), None);
        assert_eq!(parse_positive_u64("bad"), None);
    }

    #[test]
    fn rollout_budget_metadata_parses_required_fields() {
        let metadata = parse_rollout_budget_metadata(
            r#"{"schema_version":"1","service":"mnist-http","latency_budget_ms_p99":120,"error_budget_ratio":0.05,"window":"5m"}"#,
        )
        .expect("metadata must parse");
        assert_eq!(metadata.service, "mnist-http");
        assert_eq!(metadata.latency_budget_ms_p99, 120);
        assert_eq!(metadata.error_budget_ratio, 0.05);
        assert_eq!(metadata.window, "5m");
    }

    #[test]
    fn rollout_budget_metadata_rejects_invalid_error_budget() {
        let err = parse_rollout_budget_metadata(
            r#"{"schema_version":"1","service":"mnist-http","latency_budget_ms_p99":120,"error_budget_ratio":1.5,"window":"5m"}"#,
        )
        .expect_err("error budget > 1 must fail");
        assert!(err.contains("error_budget_ratio"));
    }

    #[test]
    fn rollout_budget_metadata_rejects_unknown_fields() {
        let err = parse_rollout_budget_metadata(
            r#"{"schema_version":"1","service":"mnist-http","latency_budget_ms_p99":120,"error_budget_ratio":0.05,"window":"5m","unexpected":"nope"}"#,
        )
        .expect_err("unknown fields must fail");
        assert!(err.contains("unknown field"));
    }

    #[test]
    fn rollout_budget_metadata_keeps_untrimmed_strings_per_schema() {
        let metadata = parse_rollout_budget_metadata(
            r#"{"schema_version":"1","service":"  ","latency_budget_ms_p99":120,"error_budget_ratio":0.05,"window":"  "}"#,
        )
        .expect("schema minLength allows non-empty untrimmed strings");
        assert_eq!(metadata.service, "  ");
        assert_eq!(metadata.window, "  ");
    }

    #[test]
    fn request_ids_are_monotonic() {
        let request_ids = RequestIdGenerator::new();
        assert_eq!(request_ids.next_id(), 1);
        assert_eq!(request_ids.next_id(), 2);
        assert_eq!(request_ids.next_id(), 3);
    }

    #[test]
    fn request_id_header_uses_expected_name_and_value() {
        let header = request_id_header(42);
        assert!(header.field.equiv("X-Request-Id"));
        assert_eq!(header.value.as_str(), "42");
    }

    #[test]
    fn infer_single_response_matches_golden_fixture() {
        let rows = vec![vec![0.125f32, 0.25, 0.625]];
        let actual = infer_success_payload(&rows);
        let expected = include_str!("../../fixtures/http_infer_single.json").trim();
        assert_eq!(actual, expected);
    }

    #[test]
    fn infer_batch_response_matches_golden_fixture() {
        let rows = vec![vec![0.125f32, 0.25, 0.625], vec![0.5f32, 0.25, 0.25]];
        let actual = infer_success_payload(&rows);
        let expected = include_str!("../../fixtures/http_infer_batch.json").trim();
        assert_eq!(actual, expected);
    }

    #[test]
    fn infer_saturation_returns_deterministic_503_fixture() {
        let limiter = InferConcurrencyLimiter::new(1);
        let _permit = limiter.try_acquire().expect("first permit should acquire");
        let saturated = limiter.try_acquire();
        assert!(
            saturated.is_none(),
            "second permit should fail when saturated"
        );

        let err = infer_overload_failure(1);
        assert_eq!(err.status, StatusCode(503));
        assert_eq!(err.kind, "server_overloaded");

        let actual = error_payload(err.kind, &err.detail);
        let expected = include_str!("../../fixtures/http_error_overloaded.json").trim();
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_input_error_payload_matches_fixture() {
        let actual = error_payload("invalid_input", "empty input batch");
        let expected = include_str!("../../fixtures/http_error_invalid_input.json").trim();
        assert_eq!(actual, expected);
    }

    #[test]
    fn batch_too_large_error_payload_matches_fixture() {
        let actual = error_payload(
            "batch_too_large",
            "batch size 17 exceeds HTTP_MAX_BATCH_SIZE=16",
        );
        let expected = include_str!("../../fixtures/http_error_batch_too_large.json").trim();
        assert_eq!(actual, expected);
    }

    #[test]
    fn infer_timeout_error_payload_matches_fixture() {
        let timeout = super::HttpFailure::timeout("decode", 1_500);
        let actual = error_payload(timeout.kind, &timeout.detail);
        let expected = include_str!("../../fixtures/http_error_infer_timeout.json").trim();
        assert_eq!(actual, expected);
    }
}
