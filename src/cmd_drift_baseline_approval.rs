use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const INFER_OUTPUT_DRIFT_BASELINE_APPROVAL_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_approval.json";

#[derive(Debug)]
enum DriftBaselineApprovalError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineApprovalError {}

impl From<std::io::Error> for DriftBaselineApprovalError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    baseline: PathBuf,
    approved_by: String,
    approval_ticket: String,
    approved_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineApprovalError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let baseline = load_json_object(args.baseline.as_path(), "infer output drift baseline")?;

    let approval = json!({
        "schema_version": "1",
        "baseline_path": args.baseline.display().to_string(),
        "artifact": read_string(&baseline, "artifact", args.baseline.as_path())?,
        "artifact_version": read_string(&baseline, "artifact_version", args.baseline.as_path())?,
        "policy_profile": read_string(&baseline, "policy_profile", args.baseline.as_path())?,
        "approved_by": args.approved_by,
        "approval_ticket": args.approval_ticket,
        "approved_at_unix_ms": args.approved_at_unix_ms,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&approval).map_err(|err| {
        DriftBaselineApprovalError::Parse(format!("serialize baseline approval json: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_approved",
        json!({
            "approval_path": args.out_path.display().to_string(),
            "approval": approval,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineApprovalError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut baseline = None::<PathBuf>;
    let mut approved_by = None::<String>;
    let mut approval_ticket = None::<String>;
    let mut approved_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_APPROVAL_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--baseline" => {
                let value: String = parse_value(&mut args, "--baseline")?;
                baseline = Some(PathBuf::from(value));
            }
            "--approved-by" => approved_by = Some(parse_value(&mut args, "--approved-by")?),
            "--approval-ticket" => {
                approval_ticket = Some(parse_value(&mut args, "--approval-ticket")?)
            }
            "--approved-at-unix-ms" => {
                approved_at_unix_ms = Some(parse_value(&mut args, "--approved-at-unix-ms")?)
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--baseline=") => {
                baseline = Some(PathBuf::from(
                    arg.trim_start_matches("--baseline=").to_string(),
                ))
            }
            _ if arg.starts_with("--approved-by=") => {
                approved_by = Some(arg.trim_start_matches("--approved-by=").to_string())
            }
            _ if arg.starts_with("--approval-ticket=") => {
                approval_ticket = Some(arg.trim_start_matches("--approval-ticket=").to_string())
            }
            _ if arg.starts_with("--approved-at-unix-ms=") => {
                approved_at_unix_ms = Some(
                    arg.trim_start_matches("--approved-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DriftBaselineApprovalError::InvalidArg(format!(
                                "invalid value for --approved-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DriftBaselineApprovalError::InvalidArg(format!(
                    "unknown argument for drift-approve-baseline: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        baseline: baseline.ok_or_else(|| {
            DriftBaselineApprovalError::InvalidArg(format!(
                "missing value for --baseline\n{}",
                usage()
            ))
        })?,
        approved_by: approved_by.ok_or_else(|| {
            DriftBaselineApprovalError::InvalidArg(format!(
                "missing value for --approved-by\n{}",
                usage()
            ))
        })?,
        approval_ticket: approval_ticket.ok_or_else(|| {
            DriftBaselineApprovalError::InvalidArg(format!(
                "missing value for --approval-ticket\n{}",
                usage()
            ))
        })?,
        approved_at_unix_ms: approved_at_unix_ms.ok_or_else(|| {
            DriftBaselineApprovalError::InvalidArg(format!(
                "missing value for --approved-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineApprovalError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineApprovalError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineApprovalError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DriftBaselineApprovalError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftBaselineApprovalError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DriftBaselineApprovalError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftBaselineApprovalError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftBaselineApprovalError::Parse(format!(
                "baseline `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner drift-approve-baseline --baseline PATH --approved-by NAME --approval-ticket TICKET --approved-at-unix-ms N [--out PATH]"
}
