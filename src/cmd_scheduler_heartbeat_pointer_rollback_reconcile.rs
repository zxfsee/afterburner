use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, reconciliation_status, write_emitted_json,
};
use afterburner::scheduler_heartbeat_pointer_rollback_helpers::SchedulerHeartbeatPointerRollbackRecord;
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_reconciliation.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerRollbackReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerRollbackReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerRollbackReconcileError {}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for SchedulerHeartbeatPointerRollbackReconcileError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize scheduler heartbeat pointer rollback reconciliation json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError> for SchedulerHeartbeatPointerRollbackReconcileError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    rollback: PathBuf,
    current_rollback: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerRollbackReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let rollback = load_json_object(
        args.rollback.as_path(),
        "gpu scheduler heartbeat pointer rollback",
    )?;
    let current_rollback = load_json_object(
        args.current_rollback.as_path(),
        "gpu scheduler heartbeat pointer rollback",
    )?;

    let desired =
        SchedulerHeartbeatPointerRollbackRecord::from_object(&rollback, args.rollback.as_path())?
            .to_payload();
    let current_payload = SchedulerHeartbeatPointerRollbackRecord::from_object(
        &current_rollback,
        args.current_rollback.as_path(),
    )?
    .to_payload();
    let mut current = current_payload.clone();
    current
        .as_object_mut()
        .expect("current rollback must be object")
        .insert(
            "current_rollback_path".to_string(),
            Value::from(args.current_rollback.display().to_string()),
        );

    let reconciliation_status = reconciliation_status(&desired, &current_payload);

    let reconciliation = json!({
        "schema_version": "1",
        "rollback_path": args.rollback.display().to_string(),
        "current_rollback_path": args.current_rollback.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_rollback_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatPointerRollbackReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut rollback = None::<PathBuf>;
    let mut current_rollback = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rollback" => {
                rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--rollback",
                )?))
            }
            "--current-rollback" => {
                current_rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-rollback",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--rollback=") => {
                rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--current-rollback=") => {
                current_rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--current-rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(SchedulerHeartbeatPointerRollbackReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy scheduler-heartbeat pointer rollback reconcile: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        rollback: rollback.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackReconcileError::InvalidArg(format!(
                "missing value for --rollback\n{}",
                usage()
            ))
        })?,
        current_rollback: current_rollback.ok_or_else(|| {
            SchedulerHeartbeatPointerRollbackReconcileError::InvalidArg(format!(
                "missing value for --current-rollback\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, SchedulerHeartbeatPointerRollbackReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerRollbackReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerRollbackReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy scheduler-heartbeat pointer rollback reconcile --rollback PATH --current-rollback PATH [--out PATH]"
}
