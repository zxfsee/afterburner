use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::json;

const INFER_OUTPUT_DRIFT_BASELINE_POINTER_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_pointer.json";
const INFER_OUTPUT_DRIFT_BASELINE_ROLLBACK_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_rollback.json";

#[derive(Debug)]
enum DriftBaselineRollbackError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineRollbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineRollbackError {}

impl From<std::io::Error> for DriftBaselineRollbackError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    current_pointer: PathBuf,
    restored_approval: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineRollbackError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let current_pointer = load_json_object(
        args.current_pointer.as_path(),
        "infer output drift baseline pointer",
    )?;
    let restored_approval = load_json_object(
        args.restored_approval.as_path(),
        "infer output drift baseline approval",
    )?;

    let current_policy = read_string(
        &current_pointer,
        "policy_profile",
        args.current_pointer.as_path(),
    )?;
    let restored_policy = read_string(
        &restored_approval,
        "policy_profile",
        args.restored_approval.as_path(),
    )?;
    if current_policy != restored_policy {
        return Err(DriftBaselineRollbackError::Parse(format!(
            "current pointer policy_profile `{current_policy}` does not match restored approval policy_profile `{restored_policy}`"
        )));
    }

    let restored_pointer = json!({
        "schema_version": "1",
        "approval_path": args.restored_approval.display().to_string(),
        "artifact": read_string(&restored_approval, "artifact", args.restored_approval.as_path())?,
        "artifact_version": read_string(&restored_approval, "artifact_version", args.restored_approval.as_path())?,
        "policy_profile": restored_policy,
    });

    if let Some(parent) = args.out_pointer.parent() {
        fs::create_dir_all(parent)?;
    }
    let pointer_text = serde_json::to_string_pretty(&restored_pointer).map_err(|err| {
        DriftBaselineRollbackError::Parse(format!("serialize restored pointer json: {err}"))
    })?;
    fs::write(&args.out_pointer, pointer_text)?;

    let rollback = json!({
        "schema_version": "1",
        "current_pointer_path": args.current_pointer.display().to_string(),
        "restored_approval_path": args.restored_approval.display().to_string(),
        "previous_artifact_version": read_string(&current_pointer, "artifact_version", args.current_pointer.as_path())?,
        "restored_artifact_version": read_string(&restored_approval, "artifact_version", args.restored_approval.as_path())?,
        "policy_profile": read_string(&restored_approval, "policy_profile", args.restored_approval.as_path())?,
        "rolled_back_at_unix_ms": args.rolled_back_at_unix_ms,
    });

    if let Some(parent) = args.out_record.parent() {
        fs::create_dir_all(parent)?;
    }
    let rollback_text = serde_json::to_string_pretty(&rollback).map_err(|err| {
        DriftBaselineRollbackError::Parse(format!("serialize rollback json: {err}"))
    })?;
    fs::write(&args.out_record, rollback_text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_rolled_back",
        json!({
            "pointer_path": args.out_pointer.display().to_string(),
            "rollback_path": args.out_record.display().to_string(),
            "pointer": restored_pointer,
            "rollback": rollback,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineRollbackError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut current_pointer = None::<PathBuf>;
    let mut restored_approval = None::<PathBuf>;
    let mut rolled_back_at_unix_ms = None::<u64>;
    let mut out_pointer = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_POINTER_PATH);
    let mut out_record = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_ROLLBACK_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--current-pointer" => {
                let value: String = parse_value(&mut args, "--current-pointer")?;
                current_pointer = Some(PathBuf::from(value));
            }
            "--restored-approval" => {
                let value: String = parse_value(&mut args, "--restored-approval")?;
                restored_approval = Some(PathBuf::from(value));
            }
            "--rolled-back-at-unix-ms" => {
                rolled_back_at_unix_ms = Some(parse_value(&mut args, "--rolled-back-at-unix-ms")?)
            }
            "--out-pointer" => {
                let value: String = parse_value(&mut args, "--out-pointer")?;
                out_pointer = PathBuf::from(value);
            }
            "--out-record" => {
                let value: String = parse_value(&mut args, "--out-record")?;
                out_record = PathBuf::from(value);
            }
            _ if arg.starts_with("--current-pointer=") => {
                current_pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--current-pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--restored-approval=") => {
                restored_approval = Some(PathBuf::from(
                    arg.trim_start_matches("--restored-approval=").to_string(),
                ))
            }
            _ if arg.starts_with("--rolled-back-at-unix-ms=") => {
                rolled_back_at_unix_ms = Some(
                    arg.trim_start_matches("--rolled-back-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DriftBaselineRollbackError::InvalidArg(format!(
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
                return Err(DriftBaselineRollbackError::InvalidArg(format!(
                    "unknown argument for drift rollback-approved-baseline: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        current_pointer: current_pointer.ok_or_else(|| {
            DriftBaselineRollbackError::InvalidArg(format!(
                "missing value for --current-pointer\n{}",
                usage()
            ))
        })?,
        restored_approval: restored_approval.ok_or_else(|| {
            DriftBaselineRollbackError::InvalidArg(format!(
                "missing value for --restored-approval\n{}",
                usage()
            ))
        })?,
        rolled_back_at_unix_ms: rolled_back_at_unix_ms.ok_or_else(|| {
            DriftBaselineRollbackError::InvalidArg(format!(
                "missing value for --rolled-back-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_pointer,
        out_record,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineRollbackError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineRollbackError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineRollbackError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

afterburner::define_command_input_json_kind_helpers!(
    DriftBaselineRollbackError,
    "baseline pointer"
);

fn usage() -> &'static str {
    "usage: afterburner drift rollback-approved-baseline --current-pointer PATH --restored-approval PATH --rolled-back-at-unix-ms N [--out-pointer PATH] [--out-record PATH]"
}
