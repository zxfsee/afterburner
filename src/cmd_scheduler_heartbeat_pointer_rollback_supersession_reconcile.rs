use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, reconciliation_status, write_emitted_json,
};
use afterburner::scheduler_heartbeat_pointer_rollback_helpers::SchedulerHeartbeatPointerRollbackSupersessionRecord;
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

    let desired = SchedulerHeartbeatPointerRollbackSupersessionRecord::from_object(
        &supersession,
        args.supersession.as_path(),
    )?
    .to_payload();
    let current_payload = SchedulerHeartbeatPointerRollbackSupersessionRecord::from_object(
        &current_supersession,
        args.current_supersession.as_path(),
    )?
    .to_payload();
    let mut current = current_payload.clone();
    current
        .as_object_mut()
        .expect("current supersession must be object")
        .insert(
            "current_supersession_path".to_string(),
            Value::from(args.current_supersession.display().to_string()),
        );

    let reconciliation_status = reconciliation_status(&desired, &current_payload);

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
                            "unknown argument for deploy scheduler-heartbeat pointer rollback supersession reconcile: {arg}\n{}",
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
    "usage: afterburner debug deploy scheduler-heartbeat pointer rollback supersession reconcile --supersession PATH --current-supersession PATH [--out PATH]"
}
