use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const DEPLOYMENT_VERIFICATION_RECEIPT_LOCATOR_POINTER_PATH: &str =
    "artifacts/deploy/deployment_verification_receipt_locator_pointer.json";

#[derive(Debug)]
enum DeploymentVerificationReceiptLocatorPointerError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationReceiptLocatorPointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationReceiptLocatorPointerError {}

impl From<std::io::Error> for DeploymentVerificationReceiptLocatorPointerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    locator: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationReceiptLocatorPointerError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let locator = load_json_object(
        args.locator.as_path(),
        "deployment verification receipt transport locator",
    )?;

    let pointer = json!({
        "schema_version": "1",
        "locator_path": args.locator.display().to_string(),
        "receipt_path": read_string(&locator, "receipt_path", args.locator.as_path())?,
        "artifact_version": read_string(&locator, "artifact_version", args.locator.as_path())?,
        "profile_name": read_string(&locator, "profile_name", args.locator.as_path())?,
        "verification_status": read_string(&locator, "verification_status", args.locator.as_path())?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&pointer).map_err(|err| {
        DeploymentVerificationReceiptLocatorPointerError::Parse(format!(
            "serialize deployment verification receipt locator pointer json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_verification_receipt_locator_pointer_written",
        json!({
            "pointer_path": args.out_path.display().to_string(),
            "pointer": pointer,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationReceiptLocatorPointerError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut locator = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_VERIFICATION_RECEIPT_LOCATOR_POINTER_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--locator" => {
                let value: String = parse_value(&mut args, "--locator")?;
                locator = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--locator=") => {
                locator = Some(PathBuf::from(
                    arg.trim_start_matches("--locator=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationReceiptLocatorPointerError::InvalidArg(format!(
                        "unknown argument for deploy point-verification-receipt-locator: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        locator: locator.ok_or_else(|| {
            DeploymentVerificationReceiptLocatorPointerError::InvalidArg(format!(
                "missing value for --locator\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationReceiptLocatorPointerError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationReceiptLocatorPointerError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationReceiptLocatorPointerError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentVerificationReceiptLocatorPointerError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationReceiptLocatorPointerError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentVerificationReceiptLocatorPointerError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentVerificationReceiptLocatorPointerError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentVerificationReceiptLocatorPointerError::Parse(format!(
                "deployment verification receipt transport locator `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner deploy point-verification-receipt-locator --locator PATH [--out PATH]"
}
