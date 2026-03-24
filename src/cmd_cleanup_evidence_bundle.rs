use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const ARTIFACT_CLEANUP_EVIDENCE_BUNDLE_PATH: &str =
    "artifacts/deploy/artifact_cleanup_evidence_bundle.json";

#[derive(Debug)]
enum CleanupEvidenceBundleError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for CleanupEvidenceBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CleanupEvidenceBundleError {}

impl From<std::io::Error> for CleanupEvidenceBundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for CleanupEvidenceBundleError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize cleanup evidence bundle json: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    execution_receipt: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), CleanupEvidenceBundleError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(
        args.execution_receipt.as_path(),
        "cleanup execution receipt",
    )?;
    let evidence_sources = receipt
        .get("evidence_sources")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CleanupEvidenceBundleError::Parse(format!(
                "cleanup execution receipt `{}` missing array field `evidence_sources`",
                args.execution_receipt.display()
            ))
        })?;

    for (idx, entry) in evidence_sources.iter().enumerate() {
        let object = entry.as_object().ok_or_else(|| {
            CleanupEvidenceBundleError::Parse(format!(
                "cleanup execution receipt `{}` evidence_sources[{idx}] must be an object",
                args.execution_receipt.display()
            ))
        })?;
        for key in ["artifact_path", "artifact_role"] {
            if object.get(key).and_then(Value::as_str).is_none() {
                return Err(CleanupEvidenceBundleError::Parse(format!(
                    "cleanup execution receipt `{}` evidence_sources[{idx}] missing string field `{key}`",
                    args.execution_receipt.display()
                )));
            }
        }
        if object
            .get("observed_at_unix_ms")
            .and_then(Value::as_i64)
            .is_none()
        {
            return Err(CleanupEvidenceBundleError::Parse(format!(
                "cleanup execution receipt `{}` evidence_sources[{idx}] missing integer field `observed_at_unix_ms`",
                args.execution_receipt.display()
            )));
        }
    }

    let bundle = json!({
        "schema_version": "1",
        "dry_run_receipt_path": read_string(&receipt, "dry_run_receipt_path", args.execution_receipt.as_path())?,
        "execution_receipt_path": args.execution_receipt.display().to_string(),
        "policy_profile": read_string(&receipt, "policy_profile", args.execution_receipt.as_path())?,
        "removed_count": read_array(&receipt, "removed_paths", args.execution_receipt.as_path())?.len(),
        "skipped_count": read_array(&receipt, "skipped_paths", args.execution_receipt.as_path())?.len(),
        "evidence_count": evidence_sources.len(),
        "evidence_sources": evidence_sources,
    });

    write_json_value(&args.out_path, &bundle)?;
    emit_json_artifact_written(
        "ops_cli",
        "artifact_cleanup_evidence_bundle_written",
        "bundle_path",
        &args.out_path,
        "bundle",
        bundle,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, CleanupEvidenceBundleError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut execution_receipt = None::<PathBuf>;
    let mut out_path = PathBuf::from(ARTIFACT_CLEANUP_EVIDENCE_BUNDLE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--execution-receipt" => {
                let value: String = parse_value(&mut args, "--execution-receipt")?;
                execution_receipt = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--execution-receipt=") => {
                execution_receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--execution-receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(CleanupEvidenceBundleError::InvalidArg(format!(
                    "unknown argument for cleanup evidence-bundle: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        execution_receipt: execution_receipt.ok_or_else(|| {
            CleanupEvidenceBundleError::InvalidArg(format!(
                "missing value for --execution-receipt\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, CleanupEvidenceBundleError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        CleanupEvidenceBundleError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        CleanupEvidenceBundleError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, CleanupEvidenceBundleError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        CleanupEvidenceBundleError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        CleanupEvidenceBundleError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, CleanupEvidenceBundleError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            CleanupEvidenceBundleError::Parse(format!(
                "cleanup execution receipt `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_array<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<&'a Vec<Value>, CleanupEvidenceBundleError> {
    object.get(key).and_then(Value::as_array).ok_or_else(|| {
        CleanupEvidenceBundleError::Parse(format!(
            "cleanup execution receipt `{}` missing array field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner cleanup evidence-bundle --execution-receipt PATH [--out PATH]"
}
