use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_supersession.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerRollbackSupersessionError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerRollbackSupersessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerRollbackSupersessionError {}

impl From<std::io::Error> for SchedulerHeartbeatPointerRollbackSupersessionError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    previous_rollback: PathBuf,
    next_rollback: PathBuf,
    superseded_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerRollbackSupersessionError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let previous = load_json_object(
        args.previous_rollback.as_path(),
        "gpu scheduler heartbeat pointer rollback",
    )?;
    let next = load_json_object(
        args.next_rollback.as_path(),
        "gpu scheduler heartbeat pointer rollback",
    )?;

    let previous_job_id = read_string(&previous, "job_id", args.previous_rollback.as_path())?;
    let next_job_id = read_string(&next, "job_id", args.next_rollback.as_path())?;
    if previous_job_id != next_job_id {
        return Err(SchedulerHeartbeatPointerRollbackSupersessionError::Parse(
            format!(
                "rollback records must share one job_id, found `{previous_job_id}` and `{next_job_id}`"
            ),
        ));
    }
    let previous_lease_id = read_string(&previous, "lease_id", args.previous_rollback.as_path())?;
    let next_lease_id = read_string(&next, "lease_id", args.next_rollback.as_path())?;
    if previous_lease_id != next_lease_id {
        return Err(SchedulerHeartbeatPointerRollbackSupersessionError::Parse(
            format!(
                "rollback records must share one lease_id, found `{previous_lease_id}` and `{next_lease_id}`"
            ),
        ));
    }
    let previous_worker_id = read_string(&previous, "worker_id", args.previous_rollback.as_path())?;
    let next_worker_id = read_string(&next, "worker_id", args.next_rollback.as_path())?;
    if previous_worker_id != next_worker_id {
        return Err(SchedulerHeartbeatPointerRollbackSupersessionError::Parse(
            format!(
                "rollback records must share one worker_id, found `{previous_worker_id}` and `{next_worker_id}`"
            ),
        ));
    }

    let previous_restored_pointer_path = read_string(
        &previous,
        "restored_pointer_path",
        args.previous_rollback.as_path(),
    )?;
    let next_restored_pointer_path =
        read_string(&next, "restored_pointer_path", args.next_rollback.as_path())?;
    let previous_restored_heartbeat_path = read_string(
        &previous,
        "restored_heartbeat_path",
        args.previous_rollback.as_path(),
    )?;
    let next_restored_heartbeat_path = read_string(
        &next,
        "restored_heartbeat_path",
        args.next_rollback.as_path(),
    )?;
    let previous_restored_state = read_string(
        &previous,
        "restored_state",
        args.previous_rollback.as_path(),
    )?;
    let next_restored_state = read_string(&next, "restored_state", args.next_rollback.as_path())?;
    let previous_restored_observed_at_unix_ms = read_u64(
        &previous,
        "restored_observed_at_unix_ms",
        args.previous_rollback.as_path(),
    )?;
    let next_restored_observed_at_unix_ms = read_u64(
        &next,
        "restored_observed_at_unix_ms",
        args.next_rollback.as_path(),
    )?;
    let previous_restored_progress_marker =
        read_optional_string(&previous, "restored_progress_marker");
    let next_restored_progress_marker = read_optional_string(&next, "restored_progress_marker");

    if previous_restored_pointer_path == next_restored_pointer_path
        && previous_restored_heartbeat_path == next_restored_heartbeat_path
        && previous_restored_state == next_restored_state
        && previous_restored_observed_at_unix_ms == next_restored_observed_at_unix_ms
        && previous_restored_progress_marker == next_restored_progress_marker
    {
        return Err(SchedulerHeartbeatPointerRollbackSupersessionError::Parse(
            format!(
                "next rollback `{}` must differ from previous rollback `{}` in restored_pointer_path, restored_heartbeat_path, restored_state, restored_observed_at_unix_ms, or restored_progress_marker",
                args.next_rollback.display(),
                args.previous_rollback.display()
            ),
        ));
    }

    let mut supersession = json!({
        "schema_version": "1",
        "previous_rollback_path": args.previous_rollback.display().to_string(),
        "next_rollback_path": args.next_rollback.display().to_string(),
        "previous_restored_pointer_path": previous_restored_pointer_path,
        "next_restored_pointer_path": next_restored_pointer_path,
        "previous_restored_heartbeat_path": previous_restored_heartbeat_path,
        "next_restored_heartbeat_path": next_restored_heartbeat_path,
        "job_id": next_job_id,
        "lease_id": next_lease_id,
        "worker_id": next_worker_id,
        "previous_restored_state": previous_restored_state,
        "next_restored_state": next_restored_state,
        "previous_restored_observed_at_unix_ms": previous_restored_observed_at_unix_ms,
        "next_restored_observed_at_unix_ms": next_restored_observed_at_unix_ms,
        "superseded_at_unix_ms": args.superseded_at_unix_ms,
    });
    if let Some(previous_progress_marker) = previous_restored_progress_marker {
        supersession
            .as_object_mut()
            .expect("supersession must be object")
            .insert(
                "previous_restored_progress_marker".to_string(),
                previous_progress_marker.into(),
            );
    }
    if let Some(next_progress_marker) = next_restored_progress_marker {
        supersession
            .as_object_mut()
            .expect("supersession must be object")
            .insert(
                "next_restored_progress_marker".to_string(),
                next_progress_marker.into(),
            );
    }

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&supersession).map_err(|err| {
        SchedulerHeartbeatPointerRollbackSupersessionError::Parse(format!(
            "serialize scheduler heartbeat pointer rollback supersession json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_rollback_superseded",
        json!({
            "supersession_path": args.out_path.display().to_string(),
            "supersession": supersession,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatPointerRollbackSupersessionError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut previous_rollback = None::<PathBuf>;
    let mut next_rollback = None::<PathBuf>;
    let mut superseded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--previous-rollback" => {
                previous_rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--previous-rollback",
                )?))
            }
            "--next-rollback" => {
                next_rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--next-rollback",
                )?))
            }
            "--superseded-at-unix-ms" => {
                superseded_at_unix_ms = Some(parse_value(&mut args, "--superseded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--previous-rollback=") => {
                previous_rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--previous-rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--next-rollback=") => {
                next_rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--next-rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--superseded-at-unix-ms=") => {
                superseded_at_unix_ms = Some(
                    arg.trim_start_matches("--superseded-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            SchedulerHeartbeatPointerRollbackSupersessionError::InvalidArg(format!(
                                "invalid value for --superseded-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    SchedulerHeartbeatPointerRollbackSupersessionError::InvalidArg(format!(
                        "unknown argument for deploy scheduler-heartbeat-point-rollback-supersede: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        previous_rollback: previous_rollback.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionError::InvalidArg(format!(
                "missing value for --previous-rollback\n{}",
                usage()
            ))
        })?,
        next_rollback: next_rollback.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionError::InvalidArg(format!(
                "missing value for --next-rollback\n{}",
                usage()
            ))
        })?,
        superseded_at_unix_ms: superseded_at_unix_ms.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionError::InvalidArg(format!(
                "missing value for --superseded-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, SchedulerHeartbeatPointerRollbackSupersessionError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackSupersessionError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerRollbackSupersessionError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, SchedulerHeartbeatPointerRollbackSupersessionError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        SchedulerHeartbeatPointerRollbackSupersessionError::Parse(format!(
            "parse {kind} {}: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackSupersessionError::Parse(format!(
            "{kind} {} must be a JSON object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<String, SchedulerHeartbeatPointerRollbackSupersessionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionError::Parse(format!(
                "gpu scheduler heartbeat pointer rollback {} is missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<u64, SchedulerHeartbeatPointerRollbackSupersessionError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackSupersessionError::Parse(format!(
            "gpu scheduler heartbeat pointer rollback {} is missing integer field `{key}`",
            path.display()
        ))
    })
}

fn read_optional_string(object: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy scheduler-heartbeat-point-rollback-supersede --previous-rollback PATH --next-rollback PATH --superseded-at-unix-ms MS [--out PATH]"
}
