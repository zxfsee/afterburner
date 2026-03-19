use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const INFER_OUTPUT_DRIFT_BASELINE_PATH: &str = "artifacts/eval/infer_output_drift_baseline.json";

#[derive(Debug)]
enum DriftBaselineError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineError {}

impl From<std::io::Error> for DriftBaselineError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    summary: PathBuf,
    receipt: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let summary = load_json_object(args.summary.as_path(), "infer output drift summary")?;
    let receipt = load_json_object(args.receipt.as_path(), "infer output drift receipt")?;

    let passed = receipt
        .get("passed")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            DriftBaselineError::Parse(format!(
                "infer output drift receipt `{}` missing boolean field `passed`",
                args.receipt.display()
            ))
        })?;
    if !passed {
        return Err(DriftBaselineError::Parse(format!(
            "infer output drift receipt `{}` must be passing before baseline capture",
            args.receipt.display()
        )));
    }

    let candidate_summary_path = receipt
        .get("candidate_summary_path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            DriftBaselineError::Parse(format!(
                "infer output drift receipt `{}` missing string field `candidate_summary_path`",
                args.receipt.display()
            ))
        })?;
    if Path::new(candidate_summary_path) != args.summary.as_path() {
        return Err(DriftBaselineError::Parse(format!(
            "receipt candidate_summary_path `{candidate_summary_path}` does not match requested summary `{}`",
            args.summary.display()
        )));
    }

    let baseline = json!({
        "schema_version": "1",
        "summary_path": args.summary.display().to_string(),
        "receipt_path": args.receipt.display().to_string(),
        "policy_path": read_string(&receipt, "policy_path", args.receipt.as_path())?,
        "policy_profile": read_string(&receipt, "policy_profile", args.receipt.as_path())?,
        "artifact": read_string(&summary, "artifact", args.summary.as_path())?,
        "artifact_version": read_string(&summary, "artifact_version", args.summary.as_path())?,
        "backend": read_string(&summary, "backend", args.summary.as_path())?,
        "predicted_class": read_u64(&summary, "predicted_class", args.summary.as_path())?,
        "top_probability": read_f64(&summary, "top_probability", args.summary.as_path())?,
        "margin_to_second": read_f64(&summary, "margin_to_second", args.summary.as_path())?,
        "logits_sha256": read_string(&summary, "logits_sha256", args.summary.as_path())?,
        "probabilities_sha256": read_string(&summary, "probabilities_sha256", args.summary.as_path())?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&baseline).map_err(|err| {
        DriftBaselineError::Parse(format!("serialize drift baseline json: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_written",
        json!({
            "baseline_path": args.out_path.display().to_string(),
            "baseline": baseline,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut summary = None::<PathBuf>;
    let mut receipt = None::<PathBuf>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--summary" => {
                let value: String = parse_value(&mut args, "--summary")?;
                summary = Some(PathBuf::from(value));
            }
            "--receipt" => {
                let value: String = parse_value(&mut args, "--receipt")?;
                receipt = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--summary=") => {
                summary = Some(PathBuf::from(
                    arg.trim_start_matches("--summary=").to_string(),
                ))
            }
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DriftBaselineError::InvalidArg(format!(
                    "unknown argument for drift-baseline: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        summary: summary.ok_or_else(|| {
            DriftBaselineError::InvalidArg(format!("missing value for --summary\n{}", usage()))
        })?,
        receipt: receipt.ok_or_else(|| {
            DriftBaselineError::InvalidArg(format!("missing value for --receipt\n{}", usage()))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DriftBaselineError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftBaselineError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DriftBaselineError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftBaselineError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftBaselineError::Parse(format!(
                "summary `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, DriftBaselineError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        DriftBaselineError::Parse(format!(
            "summary `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn read_f64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<f64, DriftBaselineError> {
    object.get(key).and_then(Value::as_f64).ok_or_else(|| {
        DriftBaselineError::Parse(format!(
            "summary `{}` missing number field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner drift-baseline --summary PATH --receipt PATH [--out PATH]"
}
