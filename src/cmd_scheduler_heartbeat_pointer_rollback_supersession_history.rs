use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, append_history_entry, load_json_object, load_or_init_history, read_string,
    read_u64, write_emitted_json,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_supersession_history.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerRollbackSupersessionHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerRollbackSupersessionHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerRollbackSupersessionHistoryError {}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for SchedulerHeartbeatPointerRollbackSupersessionHistoryError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize scheduler heartbeat pointer rollback supersession history json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError> for SchedulerHeartbeatPointerRollbackSupersessionHistoryError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    supersession: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerRollbackSupersessionHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let supersession = load_json_object(
        args.supersession.as_path(),
        "gpu scheduler heartbeat pointer rollback supersession",
    )?;

    let mut history = load_or_init_history(args.out_path.as_path())?;
    let mut entry = json!({
        "event": args.event,
        "supersession_path": args.supersession.display().to_string(),
        "previous_restored_pointer_path": read_string(&supersession, "previous_restored_pointer_path", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_pointer_path": read_string(&supersession, "next_restored_pointer_path", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "previous_restored_heartbeat_path": read_string(&supersession, "previous_restored_heartbeat_path", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_heartbeat_path": read_string(&supersession, "next_restored_heartbeat_path", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "job_id": read_string(&supersession, "job_id", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "lease_id": read_string(&supersession, "lease_id", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "worker_id": read_string(&supersession, "worker_id", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "previous_restored_state": read_string(&supersession, "previous_restored_state", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_state": read_string(&supersession, "next_restored_state", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "previous_restored_observed_at_unix_ms": read_u64(&supersession, "previous_restored_observed_at_unix_ms", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_observed_at_unix_ms": read_u64(&supersession, "next_restored_observed_at_unix_ms", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "superseded_at_unix_ms": read_u64(&supersession, "superseded_at_unix_ms", args.supersession.as_path(), "gpu scheduler heartbeat pointer rollback supersession")?,
        "recorded_at_unix_ms": args.recorded_at_unix_ms,
    });
    if let Some(previous_progress_marker) = supersession
        .get("previous_restored_progress_marker")
        .and_then(serde_json::Value::as_str)
    {
        entry
            .as_object_mut()
            .expect("history entry must be object")
            .insert(
                "previous_restored_progress_marker".to_string(),
                previous_progress_marker.into(),
            );
    }
    if let Some(next_progress_marker) = supersession
        .get("next_restored_progress_marker")
        .and_then(serde_json::Value::as_str)
    {
        entry
            .as_object_mut()
            .expect("history entry must be object")
            .insert(
                "next_restored_progress_marker".to_string(),
                next_progress_marker.into(),
            );
    }
    append_history_entry(&mut history, entry);

    write_emitted_json(
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    )?;
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatPointerRollbackSupersessionHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut supersession = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--supersession" => {
                supersession = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--supersession",
                )?))
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--supersession=") => {
                supersession = Some(PathBuf::from(
                    arg.trim_start_matches("--supersession=").to_string(),
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
                            SchedulerHeartbeatPointerRollbackSupersessionHistoryError::InvalidArg(
                                format!("invalid value for --recorded-at-unix-ms\n{}", usage()),
                            )
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    SchedulerHeartbeatPointerRollbackSupersessionHistoryError::InvalidArg(format!(
                        "unknown argument for deploy record-scheduler-heartbeat-pointer-rollback-supersession-history: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        supersession: supersession.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionHistoryError::InvalidArg(format!(
                "missing value for --supersession\n{}",
                usage()
            ))
        })?,
        event: event.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionHistoryError::InvalidArg(format!(
                "missing value for --event\n{}",
                usage()
            ))
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionHistoryError::InvalidArg(format!(
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
) -> Result<T, SchedulerHeartbeatPointerRollbackSupersessionHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackSupersessionHistoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerRollbackSupersessionHistoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy record-scheduler-heartbeat-pointer-rollback-supersession-history --supersession PATH --event NAME --recorded-at-unix-ms MS [--out PATH]"
}
