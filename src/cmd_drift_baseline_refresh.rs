use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const INFER_OUTPUT_DRIFT_BASELINE_PATH: &str = "artifacts/eval/infer_output_drift_baseline.json";
const INFER_OUTPUT_DRIFT_BASELINE_ARCHIVE_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline.previous.json";
const INFER_OUTPUT_DRIFT_BASELINE_REFRESH_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_refresh.json";

#[derive(Debug)]
enum DriftBaselineRefreshError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineRefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineRefreshError {}

impl From<std::io::Error> for DriftBaselineRefreshError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    summary: PathBuf,
    receipt: PathBuf,
    current_baseline: PathBuf,
    out_path: PathBuf,
    archive_path: PathBuf,
    refresh_receipt_path: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineRefreshError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let summary = load_json_object(args.summary.as_path(), "infer output drift summary")?;
    let receipt = load_json_object(args.receipt.as_path(), "infer output drift receipt")?;
    let current_baseline = load_json_object(
        args.current_baseline.as_path(),
        "infer output drift baseline",
    )?;

    let passed = receipt
        .get("passed")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            DriftBaselineRefreshError::Parse(format!(
                "infer output drift receipt `{}` missing boolean field `passed`",
                args.receipt.display()
            ))
        })?;
    if !passed {
        return Err(DriftBaselineRefreshError::Parse(format!(
            "infer output drift receipt `{}` must be passing before baseline refresh",
            args.receipt.display()
        )));
    }

    let candidate_summary_path =
        read_string(&receipt, "candidate_summary_path", args.receipt.as_path())?;
    if Path::new(candidate_summary_path.as_str()) != args.summary.as_path() {
        return Err(DriftBaselineRefreshError::Parse(format!(
            "receipt candidate_summary_path `{candidate_summary_path}` does not match requested summary `{}`",
            args.summary.display()
        )));
    }

    if let Some(parent) = args.archive_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&args.current_baseline, &args.archive_path)?;

    let refreshed_baseline = json!({
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
    let baseline_text = serde_json::to_string_pretty(&refreshed_baseline).map_err(|err| {
        DriftBaselineRefreshError::Parse(format!("serialize refreshed baseline json: {err}"))
    })?;
    fs::write(&args.out_path, baseline_text)?;

    let refresh_receipt = json!({
        "schema_version": "1",
        "previous_baseline_path": args.current_baseline.display().to_string(),
        "archived_baseline_path": args.archive_path.display().to_string(),
        "refreshed_baseline_path": args.out_path.display().to_string(),
        "summary_path": args.summary.display().to_string(),
        "receipt_path": args.receipt.display().to_string(),
        "previous_artifact_version": read_string(&current_baseline, "artifact_version", args.current_baseline.as_path())?,
        "next_artifact_version": read_string(&summary, "artifact_version", args.summary.as_path())?,
        "policy_profile": read_string(&receipt, "policy_profile", args.receipt.as_path())?,
    });

    if let Some(parent) = args.refresh_receipt_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let refresh_text = serde_json::to_string_pretty(&refresh_receipt).map_err(|err| {
        DriftBaselineRefreshError::Parse(format!("serialize baseline refresh json: {err}"))
    })?;
    fs::write(&args.refresh_receipt_path, refresh_text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_refreshed",
        json!({
            "refresh_receipt_path": args.refresh_receipt_path.display().to_string(),
            "refresh_receipt": refresh_receipt,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineRefreshError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut summary = None::<PathBuf>;
    let mut receipt = None::<PathBuf>;
    let mut current_baseline = None::<PathBuf>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_PATH);
    let mut archive_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_ARCHIVE_PATH);
    let mut refresh_receipt_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_REFRESH_PATH);

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
            "--current-baseline" => {
                let value: String = parse_value(&mut args, "--current-baseline")?;
                current_baseline = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            "--archive" => {
                let value: String = parse_value(&mut args, "--archive")?;
                archive_path = PathBuf::from(value);
            }
            "--refresh-receipt" => {
                let value: String = parse_value(&mut args, "--refresh-receipt")?;
                refresh_receipt_path = PathBuf::from(value);
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
            _ if arg.starts_with("--current-baseline=") => {
                current_baseline = Some(PathBuf::from(
                    arg.trim_start_matches("--current-baseline=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ if arg.starts_with("--archive=") => {
                archive_path = PathBuf::from(arg.trim_start_matches("--archive=").to_string())
            }
            _ if arg.starts_with("--refresh-receipt=") => {
                refresh_receipt_path =
                    PathBuf::from(arg.trim_start_matches("--refresh-receipt=").to_string())
            }
            _ => {
                return Err(DriftBaselineRefreshError::InvalidArg(format!(
                    "unknown argument for drift-refresh-baseline: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        summary: summary.ok_or_else(|| {
            DriftBaselineRefreshError::InvalidArg(format!(
                "missing value for --summary\n{}",
                usage()
            ))
        })?,
        receipt: receipt.ok_or_else(|| {
            DriftBaselineRefreshError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        current_baseline: current_baseline.ok_or_else(|| {
            DriftBaselineRefreshError::InvalidArg(format!(
                "missing value for --current-baseline\n{}",
                usage()
            ))
        })?,
        out_path,
        archive_path,
        refresh_receipt_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineRefreshError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineRefreshError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineRefreshError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DriftBaselineRefreshError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftBaselineRefreshError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DriftBaselineRefreshError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftBaselineRefreshError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftBaselineRefreshError::Parse(format!(
                "json object `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, DriftBaselineRefreshError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        DriftBaselineRefreshError::Parse(format!(
            "json object `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn read_f64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<f64, DriftBaselineRefreshError> {
    object.get(key).and_then(Value::as_f64).ok_or_else(|| {
        DriftBaselineRefreshError::Parse(format!(
            "json object `{}` missing number field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner drift-refresh-baseline --summary PATH --receipt PATH --current-baseline PATH [--out PATH] [--archive PATH] [--refresh-receipt PATH]"
}
