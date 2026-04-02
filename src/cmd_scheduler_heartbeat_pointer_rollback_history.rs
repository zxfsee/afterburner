use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_history.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerRollbackHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerRollbackHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerRollbackHistoryError {}

impl From<std::io::Error> for SchedulerHeartbeatPointerRollbackHistoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for SchedulerHeartbeatPointerRollbackHistoryError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize scheduler heartbeat pointer rollback history json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    rollback: PathBuf,
    event: String,
    recorded_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerRollbackHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let rollback = load_json_object(
        args.rollback.as_path(),
        "gpu scheduler heartbeat pointer rollback",
    )?;

    let mut history = if args.out_path.exists() {
        load_history(args.out_path.as_path())?
    } else {
        json!({
            "schema_version": "1",
            "entries": []
        })
    };

    let mut entry = json!({
        "event": args.event,
        "rollback_path": args.rollback.display().to_string(),
        "current_pointer_path": read_string(&rollback, "current_pointer_path", args.rollback.as_path())?,
        "restored_pointer_path": read_string(&rollback, "restored_pointer_path", args.rollback.as_path())?,
        "previous_heartbeat_path": read_string(&rollback, "previous_heartbeat_path", args.rollback.as_path())?,
        "restored_heartbeat_path": read_string(&rollback, "restored_heartbeat_path", args.rollback.as_path())?,
        "job_id": read_string(&rollback, "job_id", args.rollback.as_path())?,
        "lease_id": read_string(&rollback, "lease_id", args.rollback.as_path())?,
        "worker_id": read_string(&rollback, "worker_id", args.rollback.as_path())?,
        "previous_state": read_string(&rollback, "previous_state", args.rollback.as_path())?,
        "restored_state": read_string(&rollback, "restored_state", args.rollback.as_path())?,
        "previous_observed_at_unix_ms": read_u64(&rollback, "previous_observed_at_unix_ms", args.rollback.as_path())?,
        "restored_observed_at_unix_ms": read_u64(&rollback, "restored_observed_at_unix_ms", args.rollback.as_path())?,
        "rolled_back_at_unix_ms": read_u64(&rollback, "rolled_back_at_unix_ms", args.rollback.as_path())?,
        "recorded_at_unix_ms": args.recorded_at_unix_ms,
    });
    if let Some(previous_progress_marker) = rollback
        .get("previous_progress_marker")
        .and_then(Value::as_str)
    {
        entry
            .as_object_mut()
            .expect("history entry must be object")
            .insert(
                "previous_progress_marker".to_string(),
                previous_progress_marker.into(),
            );
    }
    if let Some(restored_progress_marker) = rollback
        .get("restored_progress_marker")
        .and_then(Value::as_str)
    {
        entry
            .as_object_mut()
            .expect("history entry must be object")
            .insert(
                "restored_progress_marker".to_string(),
                restored_progress_marker.into(),
            );
    }

    history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("history entries must be an array")
        .push(entry);

    write_json_value(&args.out_path, &history)?;
    emit_json_artifact_written(
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_rollback_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatPointerRollbackHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut rollback = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rollback" => {
                rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--rollback",
                )?))
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--rollback=") => {
                rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--event=") => {
                event = Some(arg.trim_start_matches("--event=").to_string())
            }
            _ if arg.starts_with("--recorded-at-unix-ms=") => {
                recorded_at_unix_ms = Some(
                    arg.trim_start_matches("--recorded-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            SchedulerHeartbeatPointerRollbackHistoryError::InvalidArg(format!(
                                "invalid value for --recorded-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(SchedulerHeartbeatPointerRollbackHistoryError::InvalidArg(
                    format!(
                        "unknown argument for deploy record-scheduler-heartbeat-pointer-rollback-history: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        rollback: rollback.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackHistoryError::InvalidArg(format!(
                "missing value for --rollback\n{}",
                usage()
            ))
        })?,
        event: event.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackHistoryError::InvalidArg(format!(
                "missing value for --event\n{}",
                usage()
            ))
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackHistoryError::InvalidArg(format!(
                "missing value for --recorded-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, SchedulerHeartbeatPointerRollbackHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackHistoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerRollbackHistoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, SchedulerHeartbeatPointerRollbackHistoryError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        SchedulerHeartbeatPointerRollbackHistoryError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackHistoryError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn load_history(path: &Path) -> Result<Value, SchedulerHeartbeatPointerRollbackHistoryError> {
    let text = fs::read_to_string(path)?;
    let history: Value = serde_json::from_str(&text).map_err(|err| {
        SchedulerHeartbeatPointerRollbackHistoryError::Parse(format!(
            "parse scheduler heartbeat pointer rollback history `{}`: {err}",
            path.display()
        ))
    })?;
    history
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackHistoryError::Parse(format!(
                "scheduler heartbeat pointer rollback history `{}` must contain an `entries` array",
                path.display()
            ))
        })?;
    Ok(history)
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, SchedulerHeartbeatPointerRollbackHistoryError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackHistoryError::Parse(format!(
                "gpu scheduler heartbeat pointer rollback `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, SchedulerHeartbeatPointerRollbackHistoryError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackHistoryError::Parse(format!(
            "gpu scheduler heartbeat pointer rollback `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy record-scheduler-heartbeat-pointer-rollback-history --rollback PATH --event NAME --recorded-at-unix-ms MS [--out PATH]"
}
