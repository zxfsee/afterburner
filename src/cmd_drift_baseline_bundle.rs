use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const INFER_OUTPUT_DRIFT_BASELINE_BUNDLE_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_bundle.json";

#[derive(Debug)]
enum DriftBaselineBundleError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineBundleError {}

impl From<std::io::Error> for DriftBaselineBundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    pointer: PathBuf,
    history: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineBundleError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let pointer = load_json_object(
        args.pointer.as_path(),
        "infer output drift baseline pointer",
    )?;
    let history = load_json_object(
        args.history.as_path(),
        "infer output drift baseline history",
    )?;
    let history_entries = history
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            DriftBaselineBundleError::Parse(format!(
                "baseline history `{}` must include an array `entries` field",
                args.history.display()
            ))
        })?;

    let approval_path = read_string(&pointer, "approval_path", args.pointer.as_path())?;
    let approval_path = resolve_relative_path(args.pointer.as_path(), approval_path.as_str());
    let approval = load_json_object(
        approval_path.as_path(),
        "infer output drift baseline approval",
    )?;
    let baseline_path = read_string(&approval, "baseline_path", approval_path.as_path())?;
    let baseline_path = resolve_relative_path(approval_path.as_path(), baseline_path.as_str());
    let _baseline = load_json_object(baseline_path.as_path(), "infer output drift baseline")?;

    let bundle = json!({
        "schema_version": "1",
        "pointer_path": args.pointer.display().to_string(),
        "history_path": args.history.display().to_string(),
        "approval_path": approval_path.display().to_string(),
        "baseline_path": baseline_path.display().to_string(),
        "artifact": read_string(&pointer, "artifact", args.pointer.as_path())?,
        "artifact_version": read_string(&pointer, "artifact_version", args.pointer.as_path())?,
        "policy_profile": read_string(&pointer, "policy_profile", args.pointer.as_path())?,
        "history_entry_count": history_entries.len(),
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&bundle).map_err(|err| {
        DriftBaselineBundleError::Parse(format!("serialize baseline bundle json: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_bundle_written",
        json!({
            "bundle_path": args.out_path.display().to_string(),
            "bundle": bundle,
        }),
    );
    Ok(())
}

fn resolve_relative_path(anchor: &Path, candidate: &str) -> PathBuf {
    let candidate = PathBuf::from(candidate);
    if candidate.is_absolute() {
        return candidate;
    }
    anchor
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(candidate)
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineBundleError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut pointer = None::<PathBuf>;
    let mut history = None::<PathBuf>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_BUNDLE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pointer" => {
                let value: String = parse_value(&mut args, "--pointer")?;
                pointer = Some(PathBuf::from(value));
            }
            "--history" => {
                let value: String = parse_value(&mut args, "--history")?;
                history = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--pointer=") => {
                pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--history=") => {
                history = Some(PathBuf::from(
                    arg.trim_start_matches("--history=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DriftBaselineBundleError::InvalidArg(format!(
                    "unknown argument for drift export-baseline-bundle: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        pointer: pointer.ok_or_else(|| {
            DriftBaselineBundleError::InvalidArg(format!(
                "missing value for --pointer\n{}",
                usage()
            ))
        })?,
        history: history.ok_or_else(|| {
            DriftBaselineBundleError::InvalidArg(format!(
                "missing value for --history\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineBundleError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineBundleError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineBundleError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DriftBaselineBundleError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftBaselineBundleError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DriftBaselineBundleError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftBaselineBundleError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftBaselineBundleError::Parse(format!(
                "json object `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner drift export-baseline-bundle --pointer PATH --history PATH [--out PATH]"
}
