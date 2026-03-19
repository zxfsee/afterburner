use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const INFER_OUTPUT_DRIFT_RECEIPT_PATH: &str = "artifacts/eval/infer_output_drift_receipt.json";

#[derive(Debug)]
enum DriftReceiptError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftReceiptError {}

impl From<std::io::Error> for DriftReceiptError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Args {
    candidate: PathBuf,
    current: PathBuf,
    policy: PathBuf,
    out_path: PathBuf,
}

#[derive(Debug, Clone)]
struct DriftSummary {
    artifact: String,
    artifact_version: String,
    predicted_class: u64,
    top_probability: f64,
    margin_to_second: f64,
}

#[derive(Debug, Clone)]
struct DriftPolicy {
    profile_name: String,
    max_top_probability_delta: f64,
    max_margin_to_second_delta: f64,
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
        Ok(passed) => {
            if passed {
                0
            } else {
                2
            }
        }
        Err(err) => {
            eprintln!("{err}");
            2
        }
    }
}

fn run_inner<I>(args: I) -> Result<bool, DriftReceiptError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let candidate = load_summary(args.candidate.as_path())?;
    let current = load_summary(args.current.as_path())?;
    let policy = load_policy(args.policy.as_path())?;

    let top_probability_delta = (candidate.top_probability - current.top_probability).abs();
    let margin_to_second_delta = (candidate.margin_to_second - current.margin_to_second).abs();
    let passed = candidate.predicted_class == current.predicted_class
        && top_probability_delta <= policy.max_top_probability_delta
        && margin_to_second_delta <= policy.max_margin_to_second_delta;

    let receipt = json!({
        "schema_version": "1",
        "candidate_summary_path": args.candidate.display().to_string(),
        "current_summary_path": args.current.display().to_string(),
        "policy_path": args.policy.display().to_string(),
        "policy_profile": policy.profile_name,
        "candidate_artifact": candidate.artifact,
        "current_artifact": current.artifact,
        "candidate_artifact_version": candidate.artifact_version,
        "current_artifact_version": current.artifact_version,
        "candidate_predicted_class": candidate.predicted_class,
        "current_predicted_class": current.predicted_class,
        "top_probability_delta": top_probability_delta,
        "margin_to_second_delta": margin_to_second_delta,
        "max_top_probability_delta": policy.max_top_probability_delta,
        "max_margin_to_second_delta": policy.max_margin_to_second_delta,
        "passed": passed
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&receipt)
        .map_err(|err| DriftReceiptError::Parse(format!("serialize drift receipt json: {err}")))?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_receipt_written",
        json!({
            "receipt_path": args.out_path.display().to_string(),
            "receipt": receipt,
        }),
    );

    Ok(passed)
}

fn load_summary(path: &Path) -> Result<DriftSummary, DriftReceiptError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftReceiptError::Parse(format!(
            "parse infer output drift summary `{}`: {err}",
            path.display()
        ))
    })?;
    let object = value.as_object().ok_or_else(|| {
        DriftReceiptError::Parse(format!(
            "infer output drift summary `{}` must be an object",
            path.display()
        ))
    })?;

    Ok(DriftSummary {
        artifact: read_string(object, "artifact", path)?,
        artifact_version: read_string(object, "artifact_version", path)?,
        predicted_class: read_u64(object, "predicted_class", path)?,
        top_probability: read_f64(object, "top_probability", path)?,
        margin_to_second: read_f64(object, "margin_to_second", path)?,
    })
}

fn load_policy(path: &Path) -> Result<DriftPolicy, DriftReceiptError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DriftReceiptError::Parse(format!(
            "parse infer output drift policy `{}`: {err}",
            path.display()
        ))
    })?;
    let object = value.as_object().ok_or_else(|| {
        DriftReceiptError::Parse(format!(
            "infer output drift policy `{}` must be an object",
            path.display()
        ))
    })?;

    let schema_version = read_string_policy(object, "schema_version", path)?;
    if schema_version != "1" {
        return Err(DriftReceiptError::Parse(format!(
            "infer output drift policy `{}` must use schema_version `1`",
            path.display()
        )));
    }

    Ok(DriftPolicy {
        profile_name: read_string_policy(object, "profile_name", path)?,
        max_top_probability_delta: read_f64_policy(object, "max_top_probability_delta", path)?,
        max_margin_to_second_delta: read_f64_policy(object, "max_margin_to_second_delta", path)?,
    })
}

fn parse_args<I>(args: I) -> Result<Args, DriftReceiptError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut candidate = None::<PathBuf>;
    let mut current = None::<PathBuf>;
    let mut policy = None::<PathBuf>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--candidate" => {
                let value: String = parse_value(&mut args, "--candidate")?;
                candidate = Some(PathBuf::from(value))
            }
            "--current" => {
                let value: String = parse_value(&mut args, "--current")?;
                current = Some(PathBuf::from(value))
            }
            "--policy" => {
                let value: String = parse_value(&mut args, "--policy")?;
                policy = Some(PathBuf::from(value))
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value)
            }
            _ if arg.starts_with("--candidate=") => {
                candidate = Some(PathBuf::from(
                    arg.trim_start_matches("--candidate=").to_string(),
                ))
            }
            _ if arg.starts_with("--current=") => {
                current = Some(PathBuf::from(
                    arg.trim_start_matches("--current=").to_string(),
                ))
            }
            _ if arg.starts_with("--policy=") => {
                policy = Some(PathBuf::from(
                    arg.trim_start_matches("--policy=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DriftReceiptError::InvalidArg(format!(
                    "unknown argument for drift-receipt: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        candidate: candidate.ok_or_else(|| {
            DriftReceiptError::InvalidArg(format!("missing value for --candidate\n{}", usage()))
        })?,
        current: current.ok_or_else(|| {
            DriftReceiptError::InvalidArg(format!("missing value for --current\n{}", usage()))
        })?,
        policy: policy.ok_or_else(|| {
            DriftReceiptError::InvalidArg(format!("missing value for --policy\n{}", usage()))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftReceiptError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftReceiptError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftReceiptError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftReceiptError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftReceiptError::Parse(format!(
                "infer output drift summary `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, DriftReceiptError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        DriftReceiptError::Parse(format!(
            "infer output drift summary `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn read_f64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<f64, DriftReceiptError> {
    object.get(key).and_then(Value::as_f64).ok_or_else(|| {
        DriftReceiptError::Parse(format!(
            "infer output drift summary `{}` missing number field `{key}`",
            path.display()
        ))
    })
}

fn read_string_policy(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DriftReceiptError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DriftReceiptError::Parse(format!(
                "infer output drift policy `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_f64_policy(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<f64, DriftReceiptError> {
    object.get(key).and_then(Value::as_f64).ok_or_else(|| {
        DriftReceiptError::Parse(format!(
            "infer output drift policy `{}` missing number field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner drift-receipt --candidate PATH --current PATH --policy PATH [--out PATH]"
}

#[cfg(test)]
mod tests {
    use super::{Args, parse_args};
    use std::path::PathBuf;

    #[test]
    fn parse_args_accepts_required_drift_receipt_flags() {
        let args = vec![
            "--candidate".to_string(),
            "candidate.json".to_string(),
            "--current=current.json".to_string(),
            "--policy".to_string(),
            "policy.json".to_string(),
            "--out".to_string(),
            "receipt.json".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse drift receipt args");
        assert_eq!(
            parsed,
            Args {
                candidate: PathBuf::from("candidate.json"),
                current: PathBuf::from("current.json"),
                policy: PathBuf::from("policy.json"),
                out_path: PathBuf::from("receipt.json"),
            }
        );
    }
}
