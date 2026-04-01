use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, read_string, read_u64, write_emitted_json,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/deploy/gpu_scheduler_heartbeat_reconciliation.json";

#[derive(Debug)]
enum SchedulerHeartbeatReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatReconcileError {}

impl From<afterburner::command_artifacts::JsonArtifactError> for SchedulerHeartbeatReconcileError {
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => Self::Parse(
                format!("serialize scheduler heartbeat reconciliation json: {err}"),
            ),
        }
    }
}

impl From<ReconciliationError> for SchedulerHeartbeatReconcileError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    heartbeat: PathBuf,
    current_heartbeat: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let heartbeat = load_json_object(args.heartbeat.as_path(), "gpu scheduler heartbeat")?;
    let current_heartbeat =
        load_json_object(args.current_heartbeat.as_path(), "gpu scheduler heartbeat")?;

    let desired = heartbeat_payload(&heartbeat, args.heartbeat.as_path())?;
    let mut current = heartbeat_payload(&current_heartbeat, args.current_heartbeat.as_path())?;
    current
        .as_object_mut()
        .expect("current heartbeat must be object")
        .insert(
            "current_heartbeat_path".to_string(),
            Value::from(args.current_heartbeat.display().to_string()),
        );

    let reconciliation_status = if desired["job_id"] == current["job_id"]
        && desired["lease_id"] == current["lease_id"]
        && desired["worker_id"] == current["worker_id"]
        && desired["state"] == current["state"]
        && desired["observed_at_unix_ms"] == current["observed_at_unix_ms"]
        && desired
            .get("progress_marker")
            .cloned()
            .unwrap_or(Value::Null)
            == current
                .get("progress_marker")
                .cloned()
                .unwrap_or(Value::Null)
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "heartbeat_path": args.heartbeat.display().to_string(),
        "current_heartbeat_path": args.current_heartbeat.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "gpu_scheduler_heartbeat_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn heartbeat_payload(
    heartbeat: &serde_json::Map<String, Value>,
    path: &std::path::Path,
) -> Result<Value, SchedulerHeartbeatReconcileError> {
    let mut payload = json!({
        "job_id": read_string(heartbeat, "job_id", path, "gpu scheduler heartbeat")?,
        "lease_id": read_string(heartbeat, "lease_id", path, "gpu scheduler heartbeat")?,
        "worker_id": read_string(heartbeat, "worker_id", path, "gpu scheduler heartbeat")?,
        "state": read_string(heartbeat, "state", path, "gpu scheduler heartbeat")?,
        "observed_at_unix_ms": read_u64(heartbeat, "observed_at_unix_ms", path, "gpu scheduler heartbeat")?,
    });
    if let Some(progress_marker) = heartbeat.get("progress_marker").and_then(Value::as_str) {
        payload
            .as_object_mut()
            .expect("heartbeat payload must be object")
            .insert("progress_marker".to_string(), progress_marker.into());
    }
    Ok(payload)
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut heartbeat = None::<PathBuf>;
    let mut current_heartbeat = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--heartbeat" => {
                heartbeat = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--heartbeat",
                )?))
            }
            "--current-heartbeat" => {
                current_heartbeat = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-heartbeat",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--heartbeat=") => {
                heartbeat = Some(PathBuf::from(
                    arg.trim_start_matches("--heartbeat=").to_string(),
                ))
            }
            _ if arg.starts_with("--current-heartbeat=") => {
                current_heartbeat = Some(PathBuf::from(
                    arg.trim_start_matches("--current-heartbeat=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(SchedulerHeartbeatReconcileError::InvalidArg(format!(
                    "unknown argument for deploy scheduler-heartbeat-reconcile: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        heartbeat: heartbeat.ok_or_else(|| {
            SchedulerHeartbeatReconcileError::InvalidArg(format!(
                "missing value for --heartbeat\n{}",
                usage()
            ))
        })?,
        current_heartbeat: current_heartbeat.ok_or_else(|| {
            SchedulerHeartbeatReconcileError::InvalidArg(format!(
                "missing value for --current-heartbeat\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, SchedulerHeartbeatReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy scheduler-heartbeat-reconcile --heartbeat PATH --current-heartbeat PATH [--out PATH]"
}
