use std::path::PathBuf;

use afterburner::command_artifacts::{JsonArtifactError, write_json_value};
use afterburner::command_reconciliation::{ReconciliationError, load_json_object};
use afterburner::observability::emit_event;
use afterburner::scheduler_heartbeat_pointer_rollback_helpers::SchedulerHeartbeatPointerRollbackRecord;
use serde_json::{Value, json};

const POINTER_OUT_PATH: &str = "artifacts/deploy/gpu_scheduler_heartbeat_pointer.json";
const ROLLBACK_OUT_PATH: &str = "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerRollbackError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerRollbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerRollbackError {}

impl From<std::io::Error> for SchedulerHeartbeatPointerRollbackError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for SchedulerHeartbeatPointerRollbackError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize rollback json: {err}"))
            }
        }
    }
}

impl From<ReconciliationError> for SchedulerHeartbeatPointerRollbackError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    current_pointer: PathBuf,
    restored_pointer: PathBuf,
    rolled_back_at_unix_ms: u64,
    out_pointer: PathBuf,
    out_record: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerRollbackError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let current_pointer = load_json_object(
        args.current_pointer.as_path(),
        "gpu scheduler heartbeat pointer",
    )?;
    let restored_pointer = load_json_object(
        args.restored_pointer.as_path(),
        "gpu scheduler heartbeat pointer",
    )?;

    let rollback_record = SchedulerHeartbeatPointerRollbackRecord::from_pointer_objects(
        &current_pointer,
        args.current_pointer.as_path(),
        &restored_pointer,
        args.restored_pointer.as_path(),
        args.rolled_back_at_unix_ms,
    )
    .map_err(SchedulerHeartbeatPointerRollbackError::from)?;

    let restored_pointer_value = Value::Object(restored_pointer.clone());
    write_json_value(args.out_pointer.as_path(), &restored_pointer_value)
        .map_err(SchedulerHeartbeatPointerRollbackError::from)?;

    let rollback = json!({
        "schema_version": "1",
        "current_pointer_path": rollback_record.current_pointer_path,
        "restored_pointer_path": rollback_record.restored_pointer_path,
        "previous_heartbeat_path": rollback_record.previous_heartbeat_path,
        "restored_heartbeat_path": rollback_record.restored_heartbeat_path,
        "job_id": rollback_record.job_id,
        "lease_id": rollback_record.lease_id,
        "worker_id": rollback_record.worker_id,
        "previous_state": rollback_record.previous_state,
        "restored_state": rollback_record.restored_state,
        "previous_observed_at_unix_ms": rollback_record.previous_observed_at_unix_ms,
        "restored_observed_at_unix_ms": rollback_record.restored_observed_at_unix_ms,
        "previous_progress_marker": rollback_record.previous_progress_marker,
        "restored_progress_marker": rollback_record.restored_progress_marker,
        "rolled_back_at_unix_ms": rollback_record.rolled_back_at_unix_ms,
    });
    write_json_value(args.out_record.as_path(), &rollback)
        .map_err(SchedulerHeartbeatPointerRollbackError::from)?;

    emit_event(
        "info",
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_rolled_back",
        json!({
            "pointer_path": args.out_pointer.display().to_string(),
            "rollback_path": args.out_record.display().to_string(),
            "pointer": restored_pointer_value,
            "rollback": rollback,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatPointerRollbackError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut current_pointer = None::<PathBuf>;
    let mut restored_pointer = None::<PathBuf>;
    let mut rolled_back_at_unix_ms = None::<u64>;
    let mut out_pointer = PathBuf::from(POINTER_OUT_PATH);
    let mut out_record = PathBuf::from(ROLLBACK_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--current-pointer" => {
                current_pointer = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-pointer",
                )?))
            }
            "--restored-pointer" => {
                restored_pointer = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--restored-pointer",
                )?))
            }
            "--rolled-back-at-unix-ms" => {
                rolled_back_at_unix_ms = Some(parse_value(&mut args, "--rolled-back-at-unix-ms")?)
            }
            "--out-pointer" => {
                out_pointer = PathBuf::from(parse_value::<String, _>(&mut args, "--out-pointer")?)
            }
            "--out-record" => {
                out_record = PathBuf::from(parse_value::<String, _>(&mut args, "--out-record")?)
            }
            _ if arg.starts_with("--current-pointer=") => {
                current_pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--current-pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--restored-pointer=") => {
                restored_pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--restored-pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--rolled-back-at-unix-ms=") => {
                rolled_back_at_unix_ms = Some(
                    arg.trim_start_matches("--rolled-back-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            SchedulerHeartbeatPointerRollbackError::InvalidArg(format!(
                                "invalid value for --rolled-back-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out-pointer=") => {
                out_pointer = PathBuf::from(arg.trim_start_matches("--out-pointer=").to_string())
            }
            _ if arg.starts_with("--out-record=") => {
                out_record = PathBuf::from(arg.trim_start_matches("--out-record=").to_string())
            }
            _ => {
                return Err(SchedulerHeartbeatPointerRollbackError::InvalidArg(format!(
                    "unknown argument for deploy scheduler-heartbeat pointer rollback: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        current_pointer: current_pointer.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackError::InvalidArg(format!(
                "missing value for --current-pointer\n{}",
                usage()
            ))
        })?,
        restored_pointer: restored_pointer.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackError::InvalidArg(format!(
                "missing value for --restored-pointer\n{}",
                usage()
            ))
        })?,
        rolled_back_at_unix_ms: rolled_back_at_unix_ms.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackError::InvalidArg(format!(
                "missing value for --rolled-back-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_pointer,
        out_record,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, SchedulerHeartbeatPointerRollbackError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerRollbackError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy scheduler-heartbeat pointer rollback --current-pointer PATH --restored-pointer PATH --rolled-back-at-unix-ms N [--out-pointer PATH] [--out-record PATH]"
}
