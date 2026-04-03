use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::json;

const DEPLOYMENT_VERIFICATION_RECEIPT_TRANSPORT_LOCATOR_PATH: &str =
    "artifacts/deploy/deployment_verification_receipt_transport_locator.json";

#[derive(Debug)]
enum DeploymentVerificationReceiptTransportLocatorError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationReceiptTransportLocatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationReceiptTransportLocatorError {}

impl From<std::io::Error> for DeploymentVerificationReceiptTransportLocatorError {
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationReceiptTransportLocatorError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "deployment verification receipt")?;

    let locator = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "artifact_version": read_string(&receipt, "artifact_version", args.receipt.as_path())?,
        "profile_name": read_string(&receipt, "profile_name", args.receipt.as_path())?,
        "verification_status": read_string(&receipt, "verification_status", args.receipt.as_path())?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&locator).map_err(|err| {
        DeploymentVerificationReceiptTransportLocatorError::Parse(format!(
            "serialize deployment verification receipt transport locator json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_verification_receipt_transport_locator_written",
        json!({
            "locator_path": args.out_path.display().to_string(),
            "locator": locator,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationReceiptTransportLocatorError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_VERIFICATION_RECEIPT_TRANSPORT_LOCATOR_PATH);

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
                return Err(
                    DeploymentVerificationReceiptTransportLocatorError::InvalidArg(format!(
                        "unknown argument for deploy point-verification-receipt-transport-locator: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DeploymentVerificationReceiptTransportLocatorError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationReceiptTransportLocatorError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationReceiptTransportLocatorError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationReceiptTransportLocatorError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_kind_helpers!(
    DeploymentVerificationReceiptTransportLocatorError,
    "deployment verification receipt"
);

fn usage() -> &'static str {
    "usage: afterburner debug deploy point-verification-receipt-transport-locator --receipt PATH [--out PATH]"
}
