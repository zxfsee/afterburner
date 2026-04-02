use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, read_string, read_u64, write_emitted_json,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerRollbackSupersessionReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerRollbackSupersessionReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerRollbackSupersessionReconcileError {}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for SchedulerHeartbeatPointerRollbackSupersessionReconcileError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize scheduler heartbeat pointer rollback supersession reconciliation json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError> for SchedulerHeartbeatPointerRollbackSupersessionReconcileError {
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
    current_supersession: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerRollbackSupersessionReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let supersession = load_json_object(
        args.supersession.as_path(),
        "gpu scheduler heartbeat pointer rollback supersession",
    )?;
    let current_supersession = load_json_object(
        args.current_supersession.as_path(),
        "gpu scheduler heartbeat pointer rollback supersession",
    )?;

    let desired = supersession_payload(&supersession, args.supersession.as_path())?;
    let mut current =
        supersession_payload(&current_supersession, args.current_supersession.as_path())?;
    current
        .as_object_mut()
        .expect("current supersession must be object")
        .insert(
            "current_supersession_path".to_string(),
            Value::from(args.current_supersession.display().to_string()),
        );

    let reconciliation_status = if desired["previous_rollback_path"]
        == current["previous_rollback_path"]
        && desired["next_rollback_path"] == current["next_rollback_path"]
        && desired["previous_restored_pointer_path"] == current["previous_restored_pointer_path"]
        && desired["next_restored_pointer_path"] == current["next_restored_pointer_path"]
        && desired["previous_restored_heartbeat_path"]
            == current["previous_restored_heartbeat_path"]
        && desired["next_restored_heartbeat_path"] == current["next_restored_heartbeat_path"]
        && desired["job_id"] == current["job_id"]
        && desired["lease_id"] == current["lease_id"]
        && desired["worker_id"] == current["worker_id"]
        && desired["previous_restored_state"] == current["previous_restored_state"]
        && desired["next_restored_state"] == current["next_restored_state"]
        && desired["previous_restored_observed_at_unix_ms"]
            == current["previous_restored_observed_at_unix_ms"]
        && desired["next_restored_observed_at_unix_ms"]
            == current["next_restored_observed_at_unix_ms"]
        && desired["superseded_at_unix_ms"] == current["superseded_at_unix_ms"]
        && desired
            .get("previous_restored_progress_marker")
            .cloned()
            .unwrap_or(Value::Null)
            == current
                .get("previous_restored_progress_marker")
                .cloned()
                .unwrap_or(Value::Null)
        && desired
            .get("next_restored_progress_marker")
            .cloned()
            .unwrap_or(Value::Null)
            == current
                .get("next_restored_progress_marker")
                .cloned()
                .unwrap_or(Value::Null)
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "supersession_path": args.supersession.display().to_string(),
        "current_supersession_path": args.current_supersession.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn supersession_payload(
    supersession: &serde_json::Map<String, Value>,
    path: &std::path::Path,
) -> Result<Value, SchedulerHeartbeatPointerRollbackSupersessionReconcileError> {
    let mut payload = json!({
        "previous_rollback_path": read_string(supersession, "previous_rollback_path", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_rollback_path": read_string(supersession, "next_rollback_path", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "previous_restored_pointer_path": read_string(supersession, "previous_restored_pointer_path", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_pointer_path": read_string(supersession, "next_restored_pointer_path", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "previous_restored_heartbeat_path": read_string(supersession, "previous_restored_heartbeat_path", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_heartbeat_path": read_string(supersession, "next_restored_heartbeat_path", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "job_id": read_string(supersession, "job_id", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "lease_id": read_string(supersession, "lease_id", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "worker_id": read_string(supersession, "worker_id", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "previous_restored_state": read_string(supersession, "previous_restored_state", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_state": read_string(supersession, "next_restored_state", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "previous_restored_observed_at_unix_ms": read_u64(supersession, "previous_restored_observed_at_unix_ms", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "next_restored_observed_at_unix_ms": read_u64(supersession, "next_restored_observed_at_unix_ms", path, "gpu scheduler heartbeat pointer rollback supersession")?,
        "superseded_at_unix_ms": read_u64(supersession, "superseded_at_unix_ms", path, "gpu scheduler heartbeat pointer rollback supersession")?,
    });
    if let Some(previous_progress_marker) = supersession
        .get("previous_restored_progress_marker")
        .and_then(Value::as_str)
    {
        payload
            .as_object_mut()
            .expect("supersession payload must be object")
            .insert(
                "previous_restored_progress_marker".to_string(),
                previous_progress_marker.into(),
            );
    }
    if let Some(next_progress_marker) = supersession
        .get("next_restored_progress_marker")
        .and_then(Value::as_str)
    {
        payload
            .as_object_mut()
            .expect("supersession payload must be object")
            .insert(
                "next_restored_progress_marker".to_string(),
                next_progress_marker.into(),
            );
    }
    Ok(payload)
}

fn parse_args<I>(
    args: I,
) -> Result<Args, SchedulerHeartbeatPointerRollbackSupersessionReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut supersession = None::<PathBuf>;
    let mut current_supersession = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--supersession" => {
                supersession = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--supersession",
                )?))
            }
            "--current-supersession" => {
                current_supersession = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-supersession",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--supersession=") => {
                supersession = Some(PathBuf::from(
                    arg.trim_start_matches("--supersession=").to_string(),
                ))
            }
            _ if arg.starts_with("--current-supersession=") => {
                current_supersession = Some(PathBuf::from(
                    arg.trim_start_matches("--current-supersession=")
                        .to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    SchedulerHeartbeatPointerRollbackSupersessionReconcileError::InvalidArg(
                        format!(
                            "unknown argument for deploy reconcile-scheduler-heartbeat-pointer-rollback-supersession: {arg}\n{}",
                            usage()
                        ),
                    ),
                );
            }
        }
    }

    Ok(Args {
        supersession: supersession.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionReconcileError::InvalidArg(format!(
                "missing value for --supersession\n{}",
                usage()
            ))
        })?,
        current_supersession: current_supersession.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackSupersessionReconcileError::InvalidArg(format!(
                "missing value for --current-supersession\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, SchedulerHeartbeatPointerRollbackSupersessionReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackSupersessionReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerRollbackSupersessionReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy reconcile-scheduler-heartbeat-pointer-rollback-supersession --supersession PATH --current-supersession PATH [--out PATH]"
}
