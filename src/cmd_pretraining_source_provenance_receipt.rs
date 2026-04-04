use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const PRETRAINING_SOURCE_PROVENANCE_RECEIPT_PATH: &str =
    "artifacts/data/pretraining_source_provenance_receipt.json";

#[derive(Debug)]
enum PretrainingSourceProvenanceReceiptError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for PretrainingSourceProvenanceReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PretrainingSourceProvenanceReceiptError {}

impl From<std::io::Error> for PretrainingSourceProvenanceReceiptError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for PretrainingSourceProvenanceReceiptError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize pretraining source provenance receipt json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    approval_receipt: PathBuf,
    registry_entry_path: String,
    upstream_locator: String,
    reviewed_metadata_sha256: String,
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

fn run_inner<I>(args: I) -> Result<(), PretrainingSourceProvenanceReceiptError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = json!({
        "schema_version": "1",
        "approval_receipt_path": args.approval_receipt.display().to_string(),
        "registry_entry_path": args.registry_entry_path,
        "upstream_locator": args.upstream_locator,
        "reviewed_metadata_sha256": args.reviewed_metadata_sha256
    });

    write_json_value(&args.out_path, &receipt)?;
    emit_json_artifact_written(
        "data_cli",
        "pretraining_source_provenance_receipt_written",
        "receipt_path",
        &args.out_path,
        "receipt",
        receipt,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, PretrainingSourceProvenanceReceiptError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut approval_receipt = None::<PathBuf>;
    let mut registry_entry_path = None::<String>;
    let mut upstream_locator = None::<String>;
    let mut reviewed_metadata_sha256 = None::<String>;
    let mut out_path = PathBuf::from(PRETRAINING_SOURCE_PROVENANCE_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--approval-receipt" => {
                approval_receipt = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--approval-receipt",
                )?))
            }
            "--registry-entry-path" => {
                registry_entry_path = Some(parse_value(&mut args, "--registry-entry-path")?)
            }
            "--upstream-locator" => {
                upstream_locator = Some(parse_value(&mut args, "--upstream-locator")?)
            }
            "--reviewed-metadata-sha256" => {
                reviewed_metadata_sha256 =
                    Some(parse_value(&mut args, "--reviewed-metadata-sha256")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--approval-receipt=") => {
                approval_receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--approval-receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--registry-entry-path=") => {
                registry_entry_path =
                    Some(arg.trim_start_matches("--registry-entry-path=").to_string())
            }
            _ if arg.starts_with("--upstream-locator=") => {
                upstream_locator = Some(arg.trim_start_matches("--upstream-locator=").to_string())
            }
            _ if arg.starts_with("--reviewed-metadata-sha256=") => {
                reviewed_metadata_sha256 = Some(
                    arg.trim_start_matches("--reviewed-metadata-sha256=")
                        .to_string(),
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(PretrainingSourceProvenanceReceiptError::InvalidArg(
                    format!(
                        "unknown argument for source provenance receipt: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    let reviewed_metadata_sha256 =
        require_non_empty(reviewed_metadata_sha256, "--reviewed-metadata-sha256")?;
    if reviewed_metadata_sha256.len() != 64
        || !reviewed_metadata_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(PretrainingSourceProvenanceReceiptError::InvalidArg(
            format!(
                "--reviewed-metadata-sha256 must be 64 lowercase hex characters\n{}",
                usage()
            ),
        ));
    }

    Ok(Args {
        approval_receipt: approval_receipt.ok_or_else(|| {
            PretrainingSourceProvenanceReceiptError::InvalidArg(format!(
                "missing value for --approval-receipt\n{}",
                usage()
            ))
        })?,
        registry_entry_path: require_non_empty(registry_entry_path, "--registry-entry-path")?,
        upstream_locator: require_non_empty(upstream_locator, "--upstream-locator")?,
        reviewed_metadata_sha256,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, PretrainingSourceProvenanceReceiptError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        PretrainingSourceProvenanceReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        PretrainingSourceProvenanceReceiptError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn require_non_empty(
    value: Option<String>,
    flag: &str,
) -> Result<String, PretrainingSourceProvenanceReceiptError> {
    let value = value.ok_or_else(|| {
        PretrainingSourceProvenanceReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    if value.trim().is_empty() {
        return Err(PretrainingSourceProvenanceReceiptError::InvalidArg(
            format!("{flag} must be non-empty\n{}", usage()),
        ));
    }
    Ok(value)
}

fn usage() -> &'static str {
    "usage: afterburner source provenance receipt --approval-receipt PATH --registry-entry-path PATH --upstream-locator LOCATOR --reviewed-metadata-sha256 SHA256 [--out PATH]"
}
