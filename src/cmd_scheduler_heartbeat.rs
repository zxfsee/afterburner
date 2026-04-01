use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str = "artifacts/deploy/gpu_scheduler_heartbeat.json";

#[derive(Debug)]
enum SchedulerHeartbeatError {
    InvalidArg(String),
    Io(std::io::Error),
    Serialize(serde_json::Error),
}

impl std::fmt::Display for SchedulerHeartbeatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Serialize(err) => write!(f, "serialize heartbeat json: {err}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatError {}

impl From<std::io::Error> for SchedulerHeartbeatError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for SchedulerHeartbeatError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Serialize(err),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    job_id: String,
    lease_id: String,
    worker_id: String,
    state: String,
    observed_at_unix_ms: u64,
    progress_marker: Option<String>,
    out_path: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let mut heartbeat = json!({
        "schema_version": "1",
        "job_id": args.job_id,
        "lease_id": args.lease_id,
        "worker_id": args.worker_id,
        "state": args.state,
        "observed_at_unix_ms": args.observed_at_unix_ms,
    });
    if let Some(progress_marker) = args.progress_marker {
        heartbeat
            .as_object_mut()
            .expect("heartbeat must be object")
            .insert("progress_marker".to_string(), progress_marker.into());
    }

    write_json_value(&args.out_path, &heartbeat)?;
    emit_json_artifact_written(
        "deploy_cli",
        "gpu_scheduler_heartbeat_written",
        "heartbeat_path",
        &args.out_path,
        "heartbeat",
        heartbeat,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut job_id = None::<String>;
    let mut lease_id = None::<String>;
    let mut worker_id = None::<String>;
    let mut state = None::<String>;
    let mut observed_at_unix_ms = None::<u64>;
    let mut progress_marker = None::<String>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--job-id" => job_id = Some(parse_string_value(&mut args, "--job-id")?),
            "--lease-id" => lease_id = Some(parse_string_value(&mut args, "--lease-id")?),
            "--worker-id" => worker_id = Some(parse_string_value(&mut args, "--worker-id")?),
            "--state" => state = Some(parse_string_value(&mut args, "--state")?),
            "--observed-at-unix-ms" => {
                observed_at_unix_ms = Some(parse_value(&mut args, "--observed-at-unix-ms")?)
            }
            "--progress-marker" => {
                progress_marker = Some(parse_string_value(&mut args, "--progress-marker")?)
            }
            "--out" => out_path = PathBuf::from(parse_string_value(&mut args, "--out")?),
            _ if arg.starts_with("--job-id=") => {
                job_id = Some(parse_inline_string("--job-id", &arg)?)
            }
            _ if arg.starts_with("--lease-id=") => {
                lease_id = Some(parse_inline_string("--lease-id", &arg)?)
            }
            _ if arg.starts_with("--worker-id=") => {
                worker_id = Some(parse_inline_string("--worker-id", &arg)?)
            }
            _ if arg.starts_with("--state=") => state = Some(parse_inline_string("--state", &arg)?),
            _ if arg.starts_with("--observed-at-unix-ms=") => {
                observed_at_unix_ms = Some(parse_inline_u64("--observed-at-unix-ms", &arg)?)
            }
            _ if arg.starts_with("--progress-marker=") => {
                progress_marker = Some(parse_inline_string("--progress-marker", &arg)?)
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(parse_inline_string("--out", &arg)?)
            }
            _ => {
                return Err(SchedulerHeartbeatError::InvalidArg(format!(
                    "unknown argument for deploy scheduler-heartbeat: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        job_id: require_arg(job_id, "--job-id")?,
        lease_id: require_arg(lease_id, "--lease-id")?,
        worker_id: require_arg(worker_id, "--worker-id")?,
        state: require_arg(state, "--state")?,
        observed_at_unix_ms: observed_at_unix_ms.ok_or_else(|| {
            SchedulerHeartbeatError::InvalidArg(format!(
                "missing value for --observed-at-unix-ms\n{}",
                usage()
            ))
        })?,
        progress_marker,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, SchedulerHeartbeatError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn parse_string_value<I>(args: &mut I, flag: &str) -> Result<String, SchedulerHeartbeatError>
where
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    if value.trim().is_empty() {
        return Err(SchedulerHeartbeatError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        )));
    }
    Ok(value)
}

fn parse_inline_string(flag: &str, arg: &str) -> Result<String, SchedulerHeartbeatError> {
    let value = arg
        .split_once('=')
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();
    if value.trim().is_empty() {
        return Err(SchedulerHeartbeatError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        )));
    }
    Ok(value)
}

fn parse_inline_u64(flag: &str, arg: &str) -> Result<u64, SchedulerHeartbeatError> {
    arg.split_once('=')
        .map(|(_, value)| value)
        .unwrap_or_default()
        .parse::<u64>()
        .map_err(|_| {
            SchedulerHeartbeatError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
        })
}

fn require_arg(value: Option<String>, flag: &str) -> Result<String, SchedulerHeartbeatError> {
    value.ok_or_else(|| {
        SchedulerHeartbeatError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy scheduler-heartbeat --job-id ID --lease-id ID --worker-id ID --state STATE --observed-at-unix-ms MS [--progress-marker TEXT] [--out PATH]"
}
