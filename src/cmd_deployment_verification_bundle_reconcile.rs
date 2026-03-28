use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, read_array, read_string, read_u64,
    reconciliation_status, write_emitted_json,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_reconciliation.json";

#[derive(Debug)]
enum DeploymentVerificationBundleReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleReconcileError {}

impl From<std::io::Error> for DeploymentVerificationBundleReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for DeploymentVerificationBundleReconcileError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize deployment verification evidence bundle reconciliation json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError> for DeploymentVerificationBundleReconcileError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    receipt: PathBuf,
    bundle: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "deployment verification receipt")?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "deployment verification evidence bundle",
    )?;
    let evidence_sources = read_array(
        &receipt,
        "evidence_sources",
        args.receipt.as_path(),
        "deployment verification receipt",
    )?;
    let evidence_count = evidence_sources.as_array().expect("array checked").len() as u64;

    let desired = json!({
        "receipt_path": args.receipt.display().to_string(),
        "artifact_version": read_string(&receipt, "artifact_version", args.receipt.as_path(), "deployment verification receipt")?,
        "profile_name": read_string(&receipt, "profile_name", args.receipt.as_path(), "deployment verification receipt")?,
        "verification_status": read_string(&receipt, "verification_status", args.receipt.as_path(), "deployment verification receipt")?,
        "evidence_count": Value::from(evidence_count),
        "evidence_sources": evidence_sources,
    });
    let current = json!({
        "receipt_path": read_string(&bundle, "receipt_path", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "artifact_version": read_string(&bundle, "artifact_version", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "profile_name": read_string(&bundle, "profile_name", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "verification_status": read_string(&bundle, "verification_status", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "evidence_sources": read_array(&bundle, "evidence_sources", args.bundle.as_path(), "deployment verification evidence bundle")?,
    });
    let reconciliation_status = reconciliation_status(&desired, &current);

    let reconciliation = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "bundle_path": args.bundle.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "deployment_verification_evidence_bundle_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationBundleReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut bundle = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                receipt = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--receipt",
                )?))
            }
            "--bundle" => {
                bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--bundle",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--bundle=") => {
                bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentVerificationBundleReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy reconcile-verification-bundle: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DeploymentVerificationBundleReconcileError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        bundle: bundle.ok_or_else(|| {
            DeploymentVerificationBundleReconcileError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationBundleReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy reconcile-verification-bundle --receipt PATH --bundle PATH [--out PATH]"
}
