use std::path::PathBuf;

use afterburner::command_artifacts::{JsonArtifactError, write_json_value};
use afterburner::command_reconciliation::{ReconciliationError, load_json_object};
use afterburner::observability::emit_event;
use afterburner::scheduler_heartbeat_pointer_rollback_helpers::SchedulerHeartbeatPointerRollbackSupersessionRecord;
use serde_json::json;

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

impl From<JsonArtifactError> for SchedulerHeartbeatPointerRollbackSupersessionError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize scheduler heartbeat pointer rollback supersession json: {err}"
            )),
        }
    }
}

impl From<ReconciliationError> for SchedulerHeartbeatPointerRollbackSupersessionError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
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

    let record = SchedulerHeartbeatPointerRollbackSupersessionRecord::from_rollback_objects(
        &previous,
        args.previous_rollback.as_path(),
        &next,
        args.next_rollback.as_path(),
        args.superseded_at_unix_ms,
    )
    .map_err(SchedulerHeartbeatPointerRollbackSupersessionError::from)?;

    let supersession = json!({
        "schema_version": "1",
        "previous_rollback_path": record.previous_rollback_path,
        "next_rollback_path": record.next_rollback_path,
        "previous_restored_pointer_path": record.previous_restored_pointer_path,
        "next_restored_pointer_path": record.next_restored_pointer_path,
        "previous_restored_heartbeat_path": record.previous_restored_heartbeat_path,
        "next_restored_heartbeat_path": record.next_restored_heartbeat_path,
        "job_id": record.job_id,
        "lease_id": record.lease_id,
        "worker_id": record.worker_id,
        "previous_restored_state": record.previous_restored_state,
        "next_restored_state": record.next_restored_state,
        "previous_restored_observed_at_unix_ms": record.previous_restored_observed_at_unix_ms,
        "next_restored_observed_at_unix_ms": record.next_restored_observed_at_unix_ms,
        "previous_restored_progress_marker": record.previous_restored_progress_marker,
        "next_restored_progress_marker": record.next_restored_progress_marker,
        "superseded_at_unix_ms": record.superseded_at_unix_ms,
    });
    write_json_value(args.out_path.as_path(), &supersession)
        .map_err(SchedulerHeartbeatPointerRollbackSupersessionError::from)?;

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

fn usage() -> &'static str {
    "usage: afterburner debug deploy scheduler-heartbeat-point-rollback-supersede --previous-rollback PATH --next-rollback PATH --superseded-at-unix-ms MS [--out PATH]"
}
