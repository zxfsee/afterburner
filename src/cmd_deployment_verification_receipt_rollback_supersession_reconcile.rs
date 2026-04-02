use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, read_string, read_u64, write_emitted_json,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_receipt_rollback_supersession_reconciliation.json";

#[derive(Debug)]
enum DeploymentVerificationReceiptRollbackSupersessionReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationReceiptRollbackSupersessionReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationReceiptRollbackSupersessionReconcileError {}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for DeploymentVerificationReceiptRollbackSupersessionReconcileError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize deployment verification receipt rollback supersession reconciliation json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError> for DeploymentVerificationReceiptRollbackSupersessionReconcileError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    supersession: PathBuf,
    current_supersession: PathBuf,
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
) -> Result<(), DeploymentVerificationReceiptRollbackSupersessionReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let supersession = load_json_object(
        args.supersession.as_path(),
        "deployment verification receipt rollback supersession",
    )?;
    let current_supersession = load_json_object(
        args.current_supersession.as_path(),
        "deployment verification receipt rollback supersession",
    )?;

    let desired = json!({
        "previous_rollback_path": read_string(&supersession, "previous_rollback_path", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "next_rollback_path": read_string(&supersession, "next_rollback_path", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "previous_restored_receipt_path": read_string(&supersession, "previous_restored_receipt_path", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "next_restored_receipt_path": read_string(&supersession, "next_restored_receipt_path", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "previous_restored_artifact_version": read_string(&supersession, "previous_restored_artifact_version", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "next_restored_artifact_version": read_string(&supersession, "next_restored_artifact_version", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "profile_name": read_string(&supersession, "profile_name", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "superseded_at_unix_ms": read_u64(&supersession, "superseded_at_unix_ms", args.supersession.as_path(), "deployment verification receipt rollback supersession")?,
    });
    let current = json!({
        "current_supersession_path": args.current_supersession.display().to_string(),
        "previous_rollback_path": read_string(&current_supersession, "previous_rollback_path", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "next_rollback_path": read_string(&current_supersession, "next_rollback_path", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "previous_restored_receipt_path": read_string(&current_supersession, "previous_restored_receipt_path", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "next_restored_receipt_path": read_string(&current_supersession, "next_restored_receipt_path", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "previous_restored_artifact_version": read_string(&current_supersession, "previous_restored_artifact_version", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "next_restored_artifact_version": read_string(&current_supersession, "next_restored_artifact_version", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "profile_name": read_string(&current_supersession, "profile_name", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
        "superseded_at_unix_ms": read_u64(&current_supersession, "superseded_at_unix_ms", args.current_supersession.as_path(), "deployment verification receipt rollback supersession")?,
    });
    let reconciliation_status = if desired["previous_rollback_path"]
        == current["previous_rollback_path"]
        && desired["next_rollback_path"] == current["next_rollback_path"]
        && desired["previous_restored_receipt_path"] == current["previous_restored_receipt_path"]
        && desired["next_restored_receipt_path"] == current["next_restored_receipt_path"]
        && desired["previous_restored_artifact_version"]
            == current["previous_restored_artifact_version"]
        && desired["next_restored_artifact_version"] == current["next_restored_artifact_version"]
        && desired["profile_name"] == current["profile_name"]
        && desired["superseded_at_unix_ms"] == current["superseded_at_unix_ms"]
    {
        "aligned"
    } else {
        "needs-update"
    };
    let reconciliation = json!({
        "schema_version": "1",
        "supersession_path": args.supersession.display().to_string(),
        "current_supersession_path": args.current_supersession.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "deployment_verification_receipt_rollback_supersession_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn parse_args<I>(
    args: I,
) -> Result<Args, DeploymentVerificationReceiptRollbackSupersessionReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut supersession = None::<PathBuf>;
    let mut current_supersession = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--supersession" => {
                supersession = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--supersession",
                )?))
            }
            "--current-supersession" => {
                current_supersession = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-supersession",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--supersession=") => {
                supersession = Some(PathBuf::from(
                    arg.trim_start_matches("--supersession=").to_string(),
                ))
            }
            _ if arg.starts_with("--current-supersession=") => {
                current_supersession = Some(PathBuf::from(
                    arg.trim_start_matches("--current-supersession=")
                        .to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationReceiptRollbackSupersessionReconcileError::InvalidArg(
                        format!(
                            "unknown argument for deploy reconcile-verification-receipt-rollback-supersession: {arg}\n{}",
                            usage()
                        ),
                    ),
                );
            }
        }
    }

    Ok(Args {
        supersession: supersession.ok_or_else(|| {
            DeploymentVerificationReceiptRollbackSupersessionReconcileError::InvalidArg(format!(
                "missing value for --supersession\n{}",
                usage()
            ))
        })?,
        current_supersession: current_supersession.ok_or_else(|| {
            DeploymentVerificationReceiptRollbackSupersessionReconcileError::InvalidArg(format!(
                "missing value for --current-supersession\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationReceiptRollbackSupersessionReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationReceiptRollbackSupersessionReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationReceiptRollbackSupersessionReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy reconcile-verification-receipt-rollback-supersession --supersession PATH --current-supersession PATH [--out PATH]"
}
