use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const DEPLOYMENT_VERIFICATION_EVIDENCE_BUNDLE_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle.json";

#[derive(Debug)]
enum DeploymentVerificationBundleError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleError {}

impl From<std::io::Error> for DeploymentVerificationBundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "deployment verification receipt")?;
    let evidence_sources = receipt
        .get("evidence_sources")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            DeploymentVerificationBundleError::Parse(format!(
                "deployment verification receipt `{}` missing array field `evidence_sources`",
                args.receipt.display()
            ))
        })?;

    for (idx, entry) in evidence_sources.iter().enumerate() {
        let object = entry.as_object().ok_or_else(|| {
            DeploymentVerificationBundleError::Parse(format!(
                "deployment verification receipt `{}` evidence_sources[{idx}] must be an object",
                args.receipt.display()
            ))
        })?;
        for key in ["artifact_path", "event_name"] {
            if object.get(key).and_then(Value::as_str).is_none() {
                return Err(DeploymentVerificationBundleError::Parse(format!(
                    "deployment verification receipt `{}` evidence_sources[{idx}] missing string field `{key}`",
                    args.receipt.display()
                )));
            }
        }
        if object
            .get("observed_at_unix_ms")
            .and_then(Value::as_i64)
            .is_none()
        {
            return Err(DeploymentVerificationBundleError::Parse(format!(
                "deployment verification receipt `{}` evidence_sources[{idx}] missing integer field `observed_at_unix_ms`",
                args.receipt.display()
            )));
        }
    }

    let bundle = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "artifact_version": read_string(&receipt, "artifact_version", args.receipt.as_path())?,
        "profile_name": read_string(&receipt, "profile_name", args.receipt.as_path())?,
        "verification_status": read_string(&receipt, "verification_status", args.receipt.as_path())?,
        "evidence_count": evidence_sources.len(),
        "evidence_sources": evidence_sources,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&bundle).map_err(|err| {
        DeploymentVerificationBundleError::Parse(format!(
            "serialize deployment verification evidence bundle json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_verification_evidence_bundle_written",
        json!({
            "bundle_path": args.out_path.display().to_string(),
            "bundle": bundle,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationBundleError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_VERIFICATION_EVIDENCE_BUNDLE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                let value: String = parse_value(&mut args, "--receipt")?;
                receipt = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
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
                return Err(DeploymentVerificationBundleError::InvalidArg(format!(
                    "unknown argument for deployment-verification-bundle: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DeploymentVerificationBundleError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DeploymentVerificationBundleError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentVerificationBundleError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationBundleError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentVerificationBundleError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentVerificationBundleError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentVerificationBundleError::Parse(format!(
                "deployment verification receipt `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner deployment-verification-bundle --receipt PATH [--out PATH]"
}
