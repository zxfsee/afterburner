use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use afterburner::observability::{attach_trace_fields, current_trace_context, emit_event};
use serde_json::json;

const DISTRIBUTED_LOAD_PROFILE_PATH: &str = "artifacts/deploy/distributed_load_profile.json";
const INFER_PAYLOAD: &str = include_str!("../fixtures/http_infer_single.json");

#[derive(Debug)]
enum DistributedLoadProfileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedLoadProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedLoadProfileError {}

impl From<std::io::Error> for DistributedLoadProfileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Args {
    addr: String,
    requests: usize,
    concurrency: usize,
    latency_budget_ms_p99: f64,
    error_budget_ratio: f64,
    out_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkerSummary {
    worker_index: usize,
    request_count: usize,
    success_count: usize,
    error_count: usize,
}

#[derive(Debug)]
struct WorkerRun {
    summary: WorkerSummary,
    latencies_ms: Vec<f64>,
}

pub fn run<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args: Vec<String> = args.collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return 0;
    }

    match run_inner(args.into_iter()) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            2
        }
    }
}

fn run_inner<I>(args: I) -> Result<(), DistributedLoadProfileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let trace = current_trace_context("deploy_cli");
    let payload = Arc::new(INFER_PAYLOAD.trim().as_bytes().to_vec());
    let started = Instant::now();

    let base = args.requests / args.concurrency;
    let remainder = args.requests % args.concurrency;
    let mut handles = Vec::with_capacity(args.concurrency);
    for worker_index in 0..args.concurrency {
        let addr = args.addr.clone();
        let payload = Arc::clone(&payload);
        let assigned = base + usize::from(worker_index < remainder);
        let traceparent = trace.traceparent.clone();
        let handle =
            thread::spawn(move || run_worker(worker_index, addr, payload, assigned, traceparent));
        handles.push(handle);
    }

    let mut worker_distribution = Vec::with_capacity(args.concurrency);
    let mut latencies_ms = Vec::with_capacity(args.requests);
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    for handle in handles {
        let worker = handle.join().map_err(|_| {
            DistributedLoadProfileError::Parse("distributed load worker panicked".to_string())
        })??;
        success_count += worker.summary.success_count;
        error_count += worker.summary.error_count;
        latencies_ms.extend(worker.latencies_ms);
        worker_distribution.push(worker.summary);
    }
    worker_distribution.sort_by_key(|worker| worker.worker_index);
    latencies_ms.sort_by(f64::total_cmp);

    let elapsed = started.elapsed().as_secs_f64();
    let throughput_requests_per_sec = if elapsed > 0.0 {
        args.requests as f64 / elapsed
    } else {
        0.0
    };
    let error_ratio = if args.requests > 0 {
        error_count as f64 / args.requests as f64
    } else {
        0.0
    };
    let latency_ms_p50 = percentile(&latencies_ms, 0.50);
    let latency_ms_p95 = percentile(&latencies_ms, 0.95);
    let latency_ms_p99 = percentile(&latencies_ms, 0.99);
    let passed =
        error_ratio <= args.error_budget_ratio && latency_ms_p99 <= args.latency_budget_ms_p99;

    let mut profile = json!({
        "schema_version": "1",
        "target_addr": args.addr,
        "request_count": args.requests,
        "concurrency": args.concurrency,
        "success_count": success_count,
        "error_count": error_count,
        "error_ratio": error_ratio,
        "latency_ms_p50": latency_ms_p50,
        "latency_ms_p95": latency_ms_p95,
        "latency_ms_p99": latency_ms_p99,
        "throughput_requests_per_sec": throughput_requests_per_sec,
        "latency_budget_ms_p99": args.latency_budget_ms_p99,
        "error_budget_ratio": args.error_budget_ratio,
        "passed": passed,
        "worker_distribution": worker_distribution.iter().map(|worker| {
            json!({
                "worker_index": worker.worker_index,
                "request_count": worker.request_count,
                "success_count": worker.success_count,
                "error_count": worker.error_count
            })
        }).collect::<Vec<_>>()
    });
    attach_trace_fields(
        profile
            .as_object_mut()
            .expect("distributed load profile must be an object"),
        &trace,
    );

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&profile).map_err(|err| {
        DistributedLoadProfileError::Parse(format!(
            "serialize distributed load profile json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "distributed_load_profile_written",
        json!({
            "profile_path": args.out_path.display().to_string(),
            "profile": profile,
            "traceparent": trace.traceparent,
            "trace_id": trace.trace_id,
        }),
    );
    Ok(())
}

fn run_worker(
    worker_index: usize,
    addr: String,
    payload: Arc<Vec<u8>>,
    request_count: usize,
    traceparent: String,
) -> Result<WorkerRun, DistributedLoadProfileError> {
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    let mut latencies_ms = Vec::with_capacity(request_count);

    for _ in 0..request_count {
        let started = Instant::now();
        let status = send_infer_request(addr.as_str(), payload.as_slice(), traceparent.as_str())?;
        latencies_ms.push(started.elapsed().as_secs_f64() * 1000.0);
        if (200..300).contains(&status) {
            success_count += 1;
        } else {
            error_count += 1;
        }
    }

    Ok(WorkerRun {
        summary: WorkerSummary {
            worker_index,
            request_count,
            success_count,
            error_count,
        },
        latencies_ms,
    })
}

fn send_infer_request(
    addr: &str,
    payload: &[u8],
    traceparent: &str,
) -> Result<u16, DistributedLoadProfileError> {
    let mut stream = TcpStream::connect(addr)?;
    let request = format!(
        "POST /infer HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\ntraceparent: {traceparent}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len(),
    );
    stream.write_all(request.as_bytes())?;
    stream.write_all(payload)?;
    stream.flush()?;

    let mut response = String::new();
    match stream.read_to_string(&mut response) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::ConnectionReset && !response.is_empty() => {}
        Err(err) => return Err(err.into()),
    }
    let status_line = response.lines().next().ok_or_else(|| {
        DistributedLoadProfileError::Parse("http response missing status line".to_string())
    })?;
    let code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| {
            DistributedLoadProfileError::Parse(format!(
                "http response status line malformed: {status_line}"
            ))
        })?
        .parse::<u16>()
        .map_err(|err| {
            DistributedLoadProfileError::Parse(format!(
                "parse http status code from `{status_line}`: {err}"
            ))
        })?;
    Ok(code)
}

fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let index = ((values.len() - 1) as f64 * percentile).round() as usize;
    values[index]
}

fn parse_args<I>(args: I) -> Result<Args, DistributedLoadProfileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut addr = None::<String>;
    let mut requests = None::<usize>;
    let mut concurrency = None::<usize>;
    let mut latency_budget_ms_p99 = None::<f64>;
    let mut error_budget_ratio = None::<f64>;
    let mut out_path = PathBuf::from(DISTRIBUTED_LOAD_PROFILE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--addr" => addr = Some(parse_value(&mut args, "--addr")?),
            "--requests" => requests = Some(parse_value(&mut args, "--requests")?),
            "--concurrency" => concurrency = Some(parse_value(&mut args, "--concurrency")?),
            "--latency-budget-ms-p99" => {
                latency_budget_ms_p99 = Some(parse_value(&mut args, "--latency-budget-ms-p99")?)
            }
            "--error-budget-ratio" => {
                error_budget_ratio = Some(parse_value(&mut args, "--error-budget-ratio")?)
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--addr=") => {
                addr = Some(arg.trim_start_matches("--addr=").to_string())
            }
            _ if arg.starts_with("--requests=") => {
                requests = Some(
                    arg.trim_start_matches("--requests=")
                        .parse()
                        .map_err(|_| invalid_arg("--requests"))?,
                )
            }
            _ if arg.starts_with("--concurrency=") => {
                concurrency = Some(
                    arg.trim_start_matches("--concurrency=")
                        .parse()
                        .map_err(|_| invalid_arg("--concurrency"))?,
                )
            }
            _ if arg.starts_with("--latency-budget-ms-p99=") => {
                latency_budget_ms_p99 = Some(
                    arg.trim_start_matches("--latency-budget-ms-p99=")
                        .parse()
                        .map_err(|_| invalid_arg("--latency-budget-ms-p99"))?,
                )
            }
            _ if arg.starts_with("--error-budget-ratio=") => {
                error_budget_ratio = Some(
                    arg.trim_start_matches("--error-budget-ratio=")
                        .parse()
                        .map_err(|_| invalid_arg("--error-budget-ratio"))?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DistributedLoadProfileError::InvalidArg(format!(
                    "unknown argument for distributed-load-profile: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    let requests = requests.ok_or_else(|| missing_arg("--requests"))?;
    let concurrency = concurrency.ok_or_else(|| missing_arg("--concurrency"))?;
    if requests == 0 || concurrency == 0 {
        return Err(DistributedLoadProfileError::InvalidArg(format!(
            "--requests and --concurrency must be greater than 0\n{}",
            usage()
        )));
    }

    Ok(Args {
        addr: require_non_empty(addr, "--addr")?,
        requests,
        concurrency,
        latency_budget_ms_p99: latency_budget_ms_p99
            .ok_or_else(|| missing_arg("--latency-budget-ms-p99"))?,
        error_budget_ratio: error_budget_ratio
            .ok_or_else(|| missing_arg("--error-budget-ratio"))?,
        out_path,
    })
}

fn require_non_empty(
    value: Option<String>,
    flag: &str,
) -> Result<String, DistributedLoadProfileError> {
    let value = value.ok_or_else(|| missing_arg(flag))?;
    if value.trim().is_empty() {
        return Err(DistributedLoadProfileError::InvalidArg(format!(
            "{flag} must be non-empty\n{}",
            usage()
        )));
    }
    Ok(value)
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DistributedLoadProfileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| missing_arg(flag))?;
    value.parse::<T>().map_err(|_| invalid_arg(flag))
}

fn missing_arg(flag: &str) -> DistributedLoadProfileError {
    DistributedLoadProfileError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
}

fn invalid_arg(flag: &str) -> DistributedLoadProfileError {
    DistributedLoadProfileError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
}

fn usage() -> &'static str {
    "usage: afterburner distributed-load-profile --addr HOST:PORT --requests N --concurrency N --latency-budget-ms-p99 F64 --error-budget-ratio F64 [--out PATH]"
}
