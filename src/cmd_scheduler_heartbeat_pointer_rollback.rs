use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
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

    let current_job_id = read_string(&current_pointer, "job_id", args.current_pointer.as_path())?;
    let restored_job_id =
        read_string(&restored_pointer, "job_id", args.restored_pointer.as_path())?;
    if current_job_id != restored_job_id {
        return Err(SchedulerHeartbeatPointerRollbackError::Parse(format!(
            "current pointer job_id `{current_job_id}` does not match restored pointer job_id `{restored_job_id}`"
        )));
    }
    let current_lease_id =
        read_string(&current_pointer, "lease_id", args.current_pointer.as_path())?;
    let restored_lease_id = read_string(
        &restored_pointer,
        "lease_id",
        args.restored_pointer.as_path(),
    )?;
    if current_lease_id != restored_lease_id {
        return Err(SchedulerHeartbeatPointerRollbackError::Parse(format!(
            "current pointer lease_id `{current_lease_id}` does not match restored pointer lease_id `{restored_lease_id}`"
        )));
    }
    let current_worker_id = read_string(
        &current_pointer,
        "worker_id",
        args.current_pointer.as_path(),
    )?;
    let restored_worker_id = read_string(
        &restored_pointer,
        "worker_id",
        args.restored_pointer.as_path(),
    )?;
    if current_worker_id != restored_worker_id {
        return Err(SchedulerHeartbeatPointerRollbackError::Parse(format!(
            "current pointer worker_id `{current_worker_id}` does not match restored pointer worker_id `{restored_worker_id}`"
        )));
    }

    let restored_pointer_value = Value::Object(restored_pointer.clone());
    write_json(
        args.out_pointer.as_path(),
        &restored_pointer_value,
        "restored pointer",
    )?;

    let mut rollback = json!({
        "schema_version": "1",
        "current_pointer_path": args.current_pointer.display().to_string(),
        "restored_pointer_path": args.restored_pointer.display().to_string(),
        "previous_heartbeat_path": read_string(&current_pointer, "heartbeat_path", args.current_pointer.as_path())?,
        "restored_heartbeat_path": read_string(&restored_pointer, "heartbeat_path", args.restored_pointer.as_path())?,
        "job_id": restored_job_id,
        "lease_id": restored_lease_id,
        "worker_id": restored_worker_id,
        "previous_state": read_string(&current_pointer, "state", args.current_pointer.as_path())?,
        "restored_state": read_string(&restored_pointer, "state", args.restored_pointer.as_path())?,
        "previous_observed_at_unix_ms": read_u64(&current_pointer, "observed_at_unix_ms", args.current_pointer.as_path())?,
        "restored_observed_at_unix_ms": read_u64(&restored_pointer, "observed_at_unix_ms", args.restored_pointer.as_path())?,
        "rolled_back_at_unix_ms": args.rolled_back_at_unix_ms,
    });
    if let Some(previous_progress_marker) =
        read_optional_string(&current_pointer, "progress_marker")
    {
        rollback
            .as_object_mut()
            .expect("rollback must be object")
            .insert(
                "previous_progress_marker".to_string(),
                previous_progress_marker.into(),
            );
    }
    if let Some(restored_progress_marker) =
        read_optional_string(&restored_pointer, "progress_marker")
    {
        rollback
            .as_object_mut()
            .expect("rollback must be object")
            .insert(
                "restored_progress_marker".to_string(),
                restored_progress_marker.into(),
            );
    }
    write_json(args.out_record.as_path(), &rollback, "rollback")?;

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
                    "unknown argument for deploy rollback-scheduler-heartbeat-pointer: {arg}\n{}",
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

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, SchedulerHeartbeatPointerRollbackError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        SchedulerHeartbeatPointerRollbackError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, SchedulerHeartbeatPointerRollbackError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackError::Parse(format!(
                "gpu scheduler heartbeat pointer `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, SchedulerHeartbeatPointerRollbackError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackError::Parse(format!(
            "gpu scheduler heartbeat pointer `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn read_optional_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Option<String> {
    object.get(key).and_then(Value::as_str).map(str::to_string)
}

fn write_json(
    path: &Path,
    value: &Value,
    kind: &str,
) -> Result<(), SchedulerHeartbeatPointerRollbackError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|err| {
        SchedulerHeartbeatPointerRollbackError::Parse(format!("serialize {kind} json: {err}"))
    })?;
    fs::write(path, text)?;
    Ok(())
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy rollback-scheduler-heartbeat-pointer --current-pointer PATH --restored-pointer PATH --rolled-back-at-unix-ms N [--out-pointer PATH] [--out-record PATH]"
}
