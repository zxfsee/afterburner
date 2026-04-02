use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, read_string, read_u64, write_emitted_json,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/gpu_scheduler_heartbeat_pointer_reconciliation.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerReconcileError {}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for SchedulerHeartbeatPointerReconcileError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => Self::Parse(
                format!("serialize scheduler heartbeat pointer reconciliation json: {err}"),
            ),
        }
    }
}

impl From<ReconciliationError> for SchedulerHeartbeatPointerReconcileError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    pointer: PathBuf,
    current_pointer: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let pointer = load_json_object(args.pointer.as_path(), "gpu scheduler heartbeat pointer")?;
    let current_pointer = load_json_object(
        args.current_pointer.as_path(),
        "gpu scheduler heartbeat pointer",
    )?;

    let desired = pointer_payload(&pointer, args.pointer.as_path())?;
    let mut current = pointer_payload(&current_pointer, args.current_pointer.as_path())?;
    current
        .as_object_mut()
        .expect("current pointer must be object")
        .insert(
            "current_pointer_path".to_string(),
            Value::from(args.current_pointer.display().to_string()),
        );

    let reconciliation_status = if desired["heartbeat_path"] == current["heartbeat_path"]
        && desired["job_id"] == current["job_id"]
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
        "pointer_path": args.pointer.display().to_string(),
        "current_pointer_path": args.current_pointer.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn pointer_payload(
    pointer: &serde_json::Map<String, Value>,
    path: &std::path::Path,
) -> Result<Value, SchedulerHeartbeatPointerReconcileError> {
    let mut payload = json!({
        "heartbeat_path": read_string(pointer, "heartbeat_path", path, "gpu scheduler heartbeat pointer")?,
        "job_id": read_string(pointer, "job_id", path, "gpu scheduler heartbeat pointer")?,
        "lease_id": read_string(pointer, "lease_id", path, "gpu scheduler heartbeat pointer")?,
        "worker_id": read_string(pointer, "worker_id", path, "gpu scheduler heartbeat pointer")?,
        "state": read_string(pointer, "state", path, "gpu scheduler heartbeat pointer")?,
        "observed_at_unix_ms": read_u64(pointer, "observed_at_unix_ms", path, "gpu scheduler heartbeat pointer")?,
    });
    if let Some(progress_marker) = pointer.get("progress_marker").and_then(Value::as_str) {
        payload
            .as_object_mut()
            .expect("pointer payload must be object")
            .insert("progress_marker".to_string(), progress_marker.into());
    }
    Ok(payload)
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatPointerReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut pointer = None::<PathBuf>;
    let mut current_pointer = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pointer" => {
                pointer = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--pointer",
                )?))
            }
            "--current-pointer" => {
                current_pointer = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-pointer",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--pointer=") => {
                pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--current-pointer=") => {
                current_pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--current-pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(SchedulerHeartbeatPointerReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy reconcile-scheduler-heartbeat-pointer: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        pointer: pointer.ok_or_else(|| {
            SchedulerHeartbeatPointerReconcileError::InvalidArg(format!(
                "missing value for --pointer\n{}",
                usage()
            ))
        })?,
        current_pointer: current_pointer.ok_or_else(|| {
            SchedulerHeartbeatPointerReconcileError::InvalidArg(format!(
                "missing value for --current-pointer\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, SchedulerHeartbeatPointerReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy reconcile-scheduler-heartbeat-pointer --pointer PATH --current-pointer PATH [--out PATH]"
}
