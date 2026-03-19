use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const INFER_OUTPUT_DRIFT_BASELINE_HANDOFF_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_handoff.json";

#[derive(Debug)]
enum DriftBaselineHandoffError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineHandoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineHandoffError {}

impl From<std::io::Error> for DriftBaselineHandoffError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    bundle: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineHandoffError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let bundle = load_json_object(args.bundle.as_path(), "infer output drift baseline bundle")?;

    let handoff = json!({
        "schema_version": "1",
        "bundle_path": args.bundle.display().to_string(),
        "approval_path": read_string(&bundle, "approval_path", args.bundle.as_path())?,
        "baseline_path": read_string(&bundle, "baseline_path", args.bundle.as_path())?,
        "artifact": read_string(&bundle, "artifact", args.bundle.as_path())?,
        "artifact_version": read_string(&bundle, "artifact_version", args.bundle.as_path())?,
        "policy_profile": read_string(&bundle, "policy_profile", args.bundle.as_path())?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&handoff).map_err(|err| {
        DriftBaselineHandoffError::Parse(format!("serialize baseline handoff json: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_handoff_written",
        json!({
            "handoff_path": args.out_path.display().to_string(),
            "handoff": handoff,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineHandoffError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut bundle = None::<PathBuf>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_HANDOFF_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bundle" => {
                let value: String = parse_value(&mut args, "--bundle")?;
                bundle = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--bundle=") => {
                bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DriftBaselineHandoffError::InvalidArg(format!(
                    "unknown argument for drift-export-baseline-handoff: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        bundle: bundle.ok_or_else(|| {
            DriftBaselineHandoffError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineHandoffError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineHandoffError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineHandoffError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DriftBaselineHandoffError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftBaselineHandoffError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DriftBaselineHandoffError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftBaselineHandoffError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftBaselineHandoffError::Parse(format!(
                "bundle `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner drift-export-baseline-handoff --bundle PATH [--out PATH]"
}
