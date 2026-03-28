use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, read_array, read_string, read_u64,
    reconciliation_status, write_emitted_json,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_receipt_reconciliation.json";

#[derive(Debug)]
enum DeploymentVerificationReceiptReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationReceiptReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationReceiptReconcileError {}

impl From<std::io::Error> for DeploymentVerificationReceiptReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for DeploymentVerificationReceiptReconcileError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => Self::Parse(
                format!("serialize deployment verification receipt reconciliation json: {err}"),
            ),
        }
    }
}

impl From<ReconciliationError> for DeploymentVerificationReceiptReconcileError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceSource {
    artifact_path: String,
    event_name: String,
    observed_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    receipt: PathBuf,
    artifact_version: String,
    profile_name: String,
    verification_status: String,
    verified_at_unix_ms: u64,
    evidence: String,
    evidence_sources: Vec<EvidenceSource>,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationReceiptReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "deployment verification receipt")?;

    let desired = json!({
        "artifact_version": args.artifact_version,
        "profile_name": args.profile_name,
        "verification_status": args.verification_status,
        "verified_at_unix_ms": args.verified_at_unix_ms,
        "evidence": args.evidence,
        "evidence_sources": args
            .evidence_sources
            .iter()
            .map(|source| json!({
                "artifact_path": source.artifact_path,
                "event_name": source.event_name,
                "observed_at_unix_ms": source.observed_at_unix_ms
            }))
            .collect::<Vec<_>>()
    });
    let current = json!({
        "artifact_version": read_string(&receipt, "artifact_version", args.receipt.as_path(), "deployment verification receipt")?,
        "profile_name": read_string(&receipt, "profile_name", args.receipt.as_path(), "deployment verification receipt")?,
        "verification_status": read_string(&receipt, "verification_status", args.receipt.as_path(), "deployment verification receipt")?,
        "verified_at_unix_ms": read_u64(&receipt, "verified_at_unix_ms", args.receipt.as_path(), "deployment verification receipt")?,
        "evidence": read_string(&receipt, "evidence", args.receipt.as_path(), "deployment verification receipt")?,
        "evidence_sources": read_array(&receipt, "evidence_sources", args.receipt.as_path(), "deployment verification receipt")?,
    });
    let reconciliation_status = reconciliation_status(&desired, &current);

    let reconciliation = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "deployment_verification_receipt_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationReceiptReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut artifact_version = None::<String>;
    let mut profile_name = None::<String>;
    let mut verification_status = None::<String>;
    let mut verified_at_unix_ms = None::<u64>;
    let mut evidence = None::<String>;
    let mut evidence_sources = Vec::new();
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                receipt = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--receipt",
                )?))
            }
            "--artifact-version" => {
                artifact_version = Some(parse_value(&mut args, "--artifact-version")?)
            }
            "--profile-name" => profile_name = Some(parse_value(&mut args, "--profile-name")?),
            "--verification-status" => {
                verification_status = Some(parse_value(&mut args, "--verification-status")?)
            }
            "--verified-at-unix-ms" => {
                verified_at_unix_ms = Some(parse_value(&mut args, "--verified-at-unix-ms")?)
            }
            "--evidence" => evidence = Some(parse_value(&mut args, "--evidence")?),
            "--evidence-source" => {
                let value: String = parse_value(&mut args, "--evidence-source")?;
                evidence_sources.push(parse_evidence_source(value.as_str())?);
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--artifact-version=") => {
                artifact_version = Some(arg.trim_start_matches("--artifact-version=").to_string())
            }
            _ if arg.starts_with("--profile-name=") => {
                profile_name = Some(arg.trim_start_matches("--profile-name=").to_string())
            }
            _ if arg.starts_with("--verification-status=") => {
                verification_status =
                    Some(arg.trim_start_matches("--verification-status=").to_string())
            }
            _ if arg.starts_with("--verified-at-unix-ms=") => {
                verified_at_unix_ms = Some(
                    arg.trim_start_matches("--verified-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DeploymentVerificationReceiptReconcileError::InvalidArg(format!(
                                "invalid value for --verified-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--evidence=") => {
                evidence = Some(arg.trim_start_matches("--evidence=").to_string())
            }
            _ if arg.starts_with("--evidence-source=") => evidence_sources.push(
                parse_evidence_source(arg.trim_start_matches("--evidence-source="))?,
            ),
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentVerificationReceiptReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy reconcile-verification-receipt: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    if evidence_sources.is_empty() {
        return Err(DeploymentVerificationReceiptReconcileError::InvalidArg(
            format!("at least one --evidence-source is required\n{}", usage()),
        ));
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DeploymentVerificationReceiptReconcileError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        artifact_version: require_non_empty(artifact_version, "--artifact-version")?,
        profile_name: require_non_empty(profile_name, "--profile-name")?,
        verification_status: require_non_empty(verification_status, "--verification-status")?,
        verified_at_unix_ms: verified_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationReceiptReconcileError::InvalidArg(format!(
                "missing value for --verified-at-unix-ms\n{}",
                usage()
            ))
        })?,
        evidence: require_non_empty(evidence, "--evidence")?,
        evidence_sources,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationReceiptReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationReceiptReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationReceiptReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn parse_evidence_source(
    raw: &str,
) -> Result<EvidenceSource, DeploymentVerificationReceiptReconcileError> {
    let parts = raw.splitn(3, ',').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(DeploymentVerificationReceiptReconcileError::InvalidArg(
            format!(
                "invalid value for --evidence-source `{raw}`; expected ARTIFACT_PATH,EVENT_NAME,OBSERVED_AT_UNIX_MS\n{}",
                usage()
            ),
        ));
    }
    let observed_at_unix_ms = parts[2].parse::<u64>().map_err(|_| {
        DeploymentVerificationReceiptReconcileError::InvalidArg(format!(
            "invalid observed_at_unix_ms in --evidence-source `{raw}`\n{}",
            usage()
        ))
    })?;
    if parts[0].trim().is_empty() || parts[1].trim().is_empty() {
        return Err(DeploymentVerificationReceiptReconcileError::InvalidArg(
            format!(
                "invalid value for --evidence-source `{raw}`; artifact path and event name must be non-empty\n{}",
                usage()
            ),
        ));
    }
    Ok(EvidenceSource {
        artifact_path: parts[0].to_string(),
        event_name: parts[1].to_string(),
        observed_at_unix_ms,
    })
}

fn require_non_empty(
    value: Option<String>,
    flag: &str,
) -> Result<String, DeploymentVerificationReceiptReconcileError> {
    match value.map(|v| v.trim().to_string()) {
        Some(v) if !v.is_empty() => Ok(v),
        _ => Err(DeploymentVerificationReceiptReconcileError::InvalidArg(
            format!("missing value for {flag}\n{}", usage()),
        )),
    }
}

fn usage() -> &'static str {
    "usage: afterburner deploy reconcile-verification-receipt --receipt PATH --artifact-version VERSION --profile-name NAME --verification-status STATUS --verified-at-unix-ms MS --evidence TEXT --evidence-source ARTIFACT_PATH,EVENT_NAME,OBSERVED_AT_UNIX_MS [--evidence-source ...] [--out PATH]"
}
