use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const INFER_OUTPUT_DRIFT_BASELINE_SUPERSESSION_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_supersession.json";

#[derive(Debug)]
enum DriftBaselineSupersessionError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineSupersessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineSupersessionError {}

impl From<std::io::Error> for DriftBaselineSupersessionError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    previous_approval: PathBuf,
    next_approval: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineSupersessionError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let previous = load_json_object(
        args.previous_approval.as_path(),
        "infer output drift baseline approval",
    )?;
    let next = load_json_object(
        args.next_approval.as_path(),
        "infer output drift baseline approval",
    )?;

    let previous_artifact_version = read_string(
        &previous,
        "artifact_version",
        args.previous_approval.as_path(),
    )?;
    let next_artifact_version =
        read_string(&next, "artifact_version", args.next_approval.as_path())?;
    if previous_artifact_version == next_artifact_version {
        return Err(DriftBaselineSupersessionError::Parse(format!(
            "next approval `{}` must target a different artifact_version than previous approval `{}`",
            args.next_approval.display(),
            args.previous_approval.display()
        )));
    }

    let previous_policy = read_string(
        &previous,
        "policy_profile",
        args.previous_approval.as_path(),
    )?;
    let next_policy = read_string(&next, "policy_profile", args.next_approval.as_path())?;
    if previous_policy != next_policy {
        return Err(DriftBaselineSupersessionError::Parse(format!(
            "baseline approvals must share one policy_profile, found `{previous_policy}` and `{next_policy}`"
        )));
    }

    let supersession = json!({
        "schema_version": "1",
        "previous_approval_path": args.previous_approval.display().to_string(),
        "next_approval_path": args.next_approval.display().to_string(),
        "superseded_by_approval_ticket": read_string(&next, "approval_ticket", args.next_approval.as_path())?,
        "previous_artifact_version": previous_artifact_version,
        "next_artifact_version": next_artifact_version,
        "policy_profile": next_policy,
        "superseded_at_unix_ms": args.superseded_at_unix_ms,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&supersession).map_err(|err| {
        DriftBaselineSupersessionError::Parse(format!(
            "serialize baseline supersession json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_approval_superseded",
        json!({
            "supersession_path": args.out_path.display().to_string(),
            "supersession": supersession,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineSupersessionError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut previous_approval = None::<PathBuf>;
    let mut next_approval = None::<PathBuf>;
    let mut superseded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_SUPERSESSION_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--previous-approval" => {
                let value: String = parse_value(&mut args, "--previous-approval")?;
                previous_approval = Some(PathBuf::from(value));
            }
            "--next-approval" => {
                let value: String = parse_value(&mut args, "--next-approval")?;
                next_approval = Some(PathBuf::from(value));
            }
            "--superseded-at-unix-ms" => {
                superseded_at_unix_ms = Some(parse_value(&mut args, "--superseded-at-unix-ms")?)
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--previous-approval=") => {
                previous_approval = Some(PathBuf::from(
                    arg.trim_start_matches("--previous-approval=").to_string(),
                ))
            }
            _ if arg.starts_with("--next-approval=") => {
                next_approval = Some(PathBuf::from(
                    arg.trim_start_matches("--next-approval=").to_string(),
                ))
            }
            _ if arg.starts_with("--superseded-at-unix-ms=") => {
                superseded_at_unix_ms = Some(
                    arg.trim_start_matches("--superseded-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DriftBaselineSupersessionError::InvalidArg(format!(
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
                return Err(DriftBaselineSupersessionError::InvalidArg(format!(
                    "unknown argument for drift-supersede-baseline-approval: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        previous_approval: previous_approval.ok_or_else(|| {
            DriftBaselineSupersessionError::InvalidArg(format!(
                "missing value for --previous-approval\n{}",
                usage()
            ))
        })?,
        next_approval: next_approval.ok_or_else(|| {
            DriftBaselineSupersessionError::InvalidArg(format!(
                "missing value for --next-approval\n{}",
                usage()
            ))
        })?,
        superseded_at_unix_ms: superseded_at_unix_ms.ok_or_else(|| {
            DriftBaselineSupersessionError::InvalidArg(format!(
                "missing value for --superseded-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineSupersessionError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineSupersessionError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineSupersessionError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DriftBaselineSupersessionError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftBaselineSupersessionError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DriftBaselineSupersessionError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftBaselineSupersessionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftBaselineSupersessionError::Parse(format!(
                "approval `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner drift-supersede-baseline-approval --previous-approval PATH --next-approval PATH --superseded-at-unix-ms N [--out PATH]"
}
