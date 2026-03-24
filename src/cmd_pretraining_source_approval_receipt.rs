use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const PRETRAINING_SOURCE_APPROVAL_RECEIPT_PATH: &str =
    "artifacts/data/pretraining_source_approval_receipt.json";

#[derive(Debug)]
enum PretrainingSourceApprovalReceiptError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for PretrainingSourceApprovalReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PretrainingSourceApprovalReceiptError {}

impl From<std::io::Error> for PretrainingSourceApprovalReceiptError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for PretrainingSourceApprovalReceiptError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize pretraining source approval receipt json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    source: String,
    source_revision: String,
    approval_status: String,
    approved_by: String,
    approval_ticket: String,
    approved_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), PretrainingSourceApprovalReceiptError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = json!({
        "schema_version": "1",
        "source": args.source,
        "source_revision": args.source_revision,
        "approval_status": args.approval_status,
        "approved_by": args.approved_by,
        "approval_ticket": args.approval_ticket,
        "approved_at_unix_ms": args.approved_at_unix_ms
    });

    write_json_value(&args.out_path, &receipt)?;
    emit_json_artifact_written(
        "data_cli",
        "pretraining_source_approval_receipt_written",
        "receipt_path",
        &args.out_path,
        "receipt",
        receipt,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, PretrainingSourceApprovalReceiptError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut source = None::<String>;
    let mut source_revision = None::<String>;
    let mut approval_status = None::<String>;
    let mut approved_by = None::<String>;
    let mut approval_ticket = None::<String>;
    let mut approved_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(PRETRAINING_SOURCE_APPROVAL_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => source = Some(parse_value(&mut args, "--source")?),
            "--source-revision" => {
                source_revision = Some(parse_value(&mut args, "--source-revision")?)
            }
            "--approval-status" => {
                approval_status = Some(parse_value(&mut args, "--approval-status")?)
            }
            "--approved-by" => approved_by = Some(parse_value(&mut args, "--approved-by")?),
            "--approval-ticket" => {
                approval_ticket = Some(parse_value(&mut args, "--approval-ticket")?)
            }
            "--approved-at-unix-ms" => {
                approved_at_unix_ms = Some(parse_value(&mut args, "--approved-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--source=") => {
                source = Some(arg.trim_start_matches("--source=").to_string())
            }
            _ if arg.starts_with("--source-revision=") => {
                source_revision = Some(arg.trim_start_matches("--source-revision=").to_string())
            }
            _ if arg.starts_with("--approval-status=") => {
                approval_status = Some(arg.trim_start_matches("--approval-status=").to_string())
            }
            _ if arg.starts_with("--approved-by=") => {
                approved_by = Some(arg.trim_start_matches("--approved-by=").to_string())
            }
            _ if arg.starts_with("--approval-ticket=") => {
                approval_ticket = Some(arg.trim_start_matches("--approval-ticket=").to_string())
            }
            _ if arg.starts_with("--approved-at-unix-ms=") => {
                approved_at_unix_ms = Some(
                    arg.trim_start_matches("--approved-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            PretrainingSourceApprovalReceiptError::InvalidArg(format!(
                                "invalid value for --approved-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(PretrainingSourceApprovalReceiptError::InvalidArg(format!(
                    "unknown argument for source approval-receipt: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        source: require_non_empty(source, "--source")?,
        source_revision: require_non_empty(source_revision, "--source-revision")?,
        approval_status: require_non_empty(approval_status, "--approval-status")?,
        approved_by: require_non_empty(approved_by, "--approved-by")?,
        approval_ticket: require_non_empty(approval_ticket, "--approval-ticket")?,
        approved_at_unix_ms: approved_at_unix_ms.ok_or_else(|| {
            PretrainingSourceApprovalReceiptError::InvalidArg(format!(
                "missing value for --approved-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, PretrainingSourceApprovalReceiptError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        PretrainingSourceApprovalReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        PretrainingSourceApprovalReceiptError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn require_non_empty(
    value: Option<String>,
    flag: &str,
) -> Result<String, PretrainingSourceApprovalReceiptError> {
    let value = value.ok_or_else(|| {
        PretrainingSourceApprovalReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    if value.trim().is_empty() {
        return Err(PretrainingSourceApprovalReceiptError::InvalidArg(format!(
            "{flag} must be non-empty\n{}",
            usage()
        )));
    }
    Ok(value)
}

fn usage() -> &'static str {
    "usage: afterburner source approval-receipt --source NAME --source-revision REV --approval-status STATUS --approved-by NAME --approval-ticket TICKET --approved-at-unix-ms N [--out PATH]"
}
