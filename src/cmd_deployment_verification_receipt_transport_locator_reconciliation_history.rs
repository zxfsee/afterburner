use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, append_history_entry, load_json_object, load_or_init_history, read_object,
    read_string, write_emitted_json,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str = "artifacts/deploy/deployment_verification_receipt_transport_locator_reconciliation_history.json";

#[derive(Debug)]
enum DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError {}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize deployment verification receipt transport locator reconciliation history json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError>
    for DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError
{
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    reconciliation: PathBuf,
    event: String,
    recorded_at_unix_ms: u64,
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

fn run_inner<I>(
    args: I,
) -> Result<(), DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let reconciliation = load_json_object(
        args.reconciliation.as_path(),
        "deployment verification receipt transport locator reconciliation",
    )?;

    let mut history = load_or_init_history(args.out_path.as_path())?;
    let entry = json!({
        "event": args.event,
        "reconciliation_path": args.reconciliation.display().to_string(),
        "receipt_path": read_string(&reconciliation, "receipt_path", args.reconciliation.as_path(), "deployment verification receipt transport locator reconciliation")?,
        "locator_path": read_string(&reconciliation, "locator_path", args.reconciliation.as_path(), "deployment verification receipt transport locator reconciliation")?,
        "reconciliation_status": read_string(&reconciliation, "reconciliation_status", args.reconciliation.as_path(), "deployment verification receipt transport locator reconciliation")?,
        "desired": read_object(&reconciliation, "desired", args.reconciliation.as_path(), "deployment verification receipt transport locator reconciliation")?,
        "current": read_object(&reconciliation, "current", args.reconciliation.as_path(), "deployment verification receipt transport locator reconciliation")?,
        "recorded_at_unix_ms": args.recorded_at_unix_ms,
    });
    append_history_entry(&mut history, entry);

    write_emitted_json(
        "deploy_cli",
        "deployment_verification_receipt_transport_locator_reconciliation_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    )?;
    Ok(())
}

fn parse_args<I>(
    args: I,
) -> Result<Args, DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut reconciliation = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--reconciliation" => {
                reconciliation = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--reconciliation",
                )?))
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--reconciliation=") => {
                reconciliation = Some(PathBuf::from(
                    arg.trim_start_matches("--reconciliation=").to_string(),
                ))
            }
            _ if arg.starts_with("--event=") => {
                event = Some(arg.trim_start_matches("--event=").to_string())
            }
            _ if arg.starts_with("--recorded-at-unix-ms=") => {
                recorded_at_unix_ms = Some(
                    arg.trim_start_matches("--recorded-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError::InvalidArg(
                                format!("invalid value for --recorded-at-unix-ms\n{}", usage()),
                            )
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError::InvalidArg(format!(
                        "unknown argument for deploy record-verification-receipt-transport-locator-reconciliation-history: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        reconciliation: reconciliation.ok_or_else(|| {
            DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError::InvalidArg(
                format!("missing value for --reconciliation\n{}", usage()),
            )
        })?,
        event: event.ok_or_else(|| {
            DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError::InvalidArg(
                format!("missing value for --event\n{}", usage()),
            )
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError::InvalidArg(
                format!("missing value for --recorded-at-unix-ms\n{}", usage()),
            )
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError::InvalidArg(
            format!("missing value for {flag}\n{}", usage()),
        )
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationReceiptTransportLocatorReconciliationHistoryError::InvalidArg(
            format!("invalid value for {flag}\n{}", usage()),
        )
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy record-verification-receipt-transport-locator-reconciliation-history --reconciliation PATH --event NAME --recorded-at-unix-ms MS [--out PATH]"
}
