use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_receipt_transport_locator_reconciliation.json";

#[derive(Debug)]
enum DeploymentVerificationReceiptTransportLocatorReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationReceiptTransportLocatorReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationReceiptTransportLocatorReconcileError {}

impl From<std::io::Error> for DeploymentVerificationReceiptTransportLocatorReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentVerificationReceiptTransportLocatorReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment verification receipt transport locator reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    receipt: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationReceiptTransportLocatorReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "deployment verification receipt")?;
    let locator = load_json_object(
        args.locator.as_path(),
        "deployment verification receipt transport locator",
    )?;

    let desired = json!({
        "artifact_version": read_string(&receipt, "artifact_version", args.receipt.as_path(), "deployment verification receipt")?,
        "profile_name": read_string(&receipt, "profile_name", args.receipt.as_path(), "deployment verification receipt")?,
        "verification_status": read_string(&receipt, "verification_status", args.receipt.as_path(), "deployment verification receipt")?,
    });
    let current = json!({
        "receipt_path": read_string(&locator, "receipt_path", args.locator.as_path(), "deployment verification receipt transport locator")?,
        "artifact_version": read_string(&locator, "artifact_version", args.locator.as_path(), "deployment verification receipt transport locator")?,
        "profile_name": read_string(&locator, "profile_name", args.locator.as_path(), "deployment verification receipt transport locator")?,
        "verification_status": read_string(&locator, "verification_status", args.locator.as_path(), "deployment verification receipt transport locator")?,
    });
    let reconciliation_status = if desired["artifact_version"] == current["artifact_version"]
        && desired["profile_name"] == current["profile_name"]
        && desired["verification_status"] == current["verification_status"]
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "locator_path": args.locator.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_verification_receipt_transport_locator_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(
    args: I,
) -> Result<Args, DeploymentVerificationReceiptTransportLocatorReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut locator = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                receipt = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--receipt",
                )?))
            }
            "--locator" => {
                locator = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--locator",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
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
                    DeploymentVerificationReceiptTransportLocatorReconcileError::InvalidArg(
                        format!(
                            "unknown argument for deploy reconcile-verification-receipt-transport-locator: {arg}\n{}",
                            usage()
                        ),
                    ),
                );
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DeploymentVerificationReceiptTransportLocatorReconcileError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        locator: locator.ok_or_else(|| {
            DeploymentVerificationReceiptTransportLocatorReconcileError::InvalidArg(format!(
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
) -> Result<T, DeploymentVerificationReceiptTransportLocatorReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationReceiptTransportLocatorReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationReceiptTransportLocatorReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<
    serde_json::Map<String, Value>,
    DeploymentVerificationReceiptTransportLocatorReconcileError,
> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationReceiptTransportLocatorReconcileError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentVerificationReceiptTransportLocatorReconcileError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<String, DeploymentVerificationReceiptTransportLocatorReconcileError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentVerificationReceiptTransportLocatorReconcileError::Parse(format!(
                "{kind} `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy reconcile-verification-receipt-transport-locator --receipt PATH --locator PATH [--out PATH]"
}
