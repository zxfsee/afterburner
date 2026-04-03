use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_rollback_reconciliation.json";

#[derive(Debug)]
enum DeploymentVerificationBundleRollbackReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleRollbackReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleRollbackReconcileError {}

impl From<std::io::Error> for DeploymentVerificationBundleRollbackReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentVerificationBundleRollbackReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment verification bundle rollback reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    rollback: PathBuf,
    current_rollback: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleRollbackReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let rollback = load_json_object(
        args.rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;
    let current_rollback = load_json_object(
        args.current_rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;

    let desired = json!({
        "current_bundle_path": read_string(&rollback, "current_bundle_path", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_bundle_path": read_string(&rollback, "restored_bundle_path", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_receipt_path": read_string(&rollback, "previous_receipt_path", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_receipt_path": read_string(&rollback, "restored_receipt_path", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_artifact_version": read_string(&rollback, "previous_artifact_version", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_artifact_version": read_string(&rollback, "restored_artifact_version", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "profile_name": read_string(&rollback, "profile_name", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_verification_status": read_string(&rollback, "previous_verification_status", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_verification_status": read_string(&rollback, "restored_verification_status", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_evidence_count": read_u64(&rollback, "previous_evidence_count", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_evidence_count": read_u64(&rollback, "restored_evidence_count", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "rolled_back_at_unix_ms": read_u64(&rollback, "rolled_back_at_unix_ms", args.rollback.as_path(), "deployment verification evidence bundle rollback")?,
    });
    let current = json!({
        "current_rollback_path": args.current_rollback.display().to_string(),
        "current_bundle_path": read_string(&current_rollback, "current_bundle_path", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_bundle_path": read_string(&current_rollback, "restored_bundle_path", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_receipt_path": read_string(&current_rollback, "previous_receipt_path", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_receipt_path": read_string(&current_rollback, "restored_receipt_path", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_artifact_version": read_string(&current_rollback, "previous_artifact_version", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_artifact_version": read_string(&current_rollback, "restored_artifact_version", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "profile_name": read_string(&current_rollback, "profile_name", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_verification_status": read_string(&current_rollback, "previous_verification_status", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_verification_status": read_string(&current_rollback, "restored_verification_status", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_evidence_count": read_u64(&current_rollback, "previous_evidence_count", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "restored_evidence_count": read_u64(&current_rollback, "restored_evidence_count", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "rolled_back_at_unix_ms": read_u64(&current_rollback, "rolled_back_at_unix_ms", args.current_rollback.as_path(), "deployment verification evidence bundle rollback")?,
    });
    let reconciliation_status = if desired["current_bundle_path"] == current["current_bundle_path"]
        && desired["restored_bundle_path"] == current["restored_bundle_path"]
        && desired["previous_receipt_path"] == current["previous_receipt_path"]
        && desired["restored_receipt_path"] == current["restored_receipt_path"]
        && desired["previous_artifact_version"] == current["previous_artifact_version"]
        && desired["restored_artifact_version"] == current["restored_artifact_version"]
        && desired["profile_name"] == current["profile_name"]
        && desired["previous_verification_status"] == current["previous_verification_status"]
        && desired["restored_verification_status"] == current["restored_verification_status"]
        && desired["previous_evidence_count"] == current["previous_evidence_count"]
        && desired["restored_evidence_count"] == current["restored_evidence_count"]
        && desired["rolled_back_at_unix_ms"] == current["rolled_back_at_unix_ms"]
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "rollback_path": args.rollback.display().to_string(),
        "current_rollback_path": args.current_rollback.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_verification_evidence_bundle_rollback_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationBundleRollbackReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut rollback = None::<PathBuf>;
    let mut current_rollback = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rollback" => {
                rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--rollback",
                )?))
            }
            "--current-rollback" => {
                current_rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-rollback",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--rollback=") => {
                rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--current-rollback=") => {
                current_rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--current-rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationBundleRollbackReconcileError::InvalidArg(format!(
                        "unknown argument for deploy reconcile-verification-bundle-rollback: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        rollback: rollback.ok_or_else(|| {
            DeploymentVerificationBundleRollbackReconcileError::InvalidArg(format!(
                "missing value for --rollback\n{}",
                usage()
            ))
        })?,
        current_rollback: current_rollback.ok_or_else(|| {
            DeploymentVerificationBundleRollbackReconcileError::InvalidArg(format!(
                "missing value for --current-rollback\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationBundleRollbackReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleRollbackReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleRollbackReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(DeploymentVerificationBundleRollbackReconcileError);

fn usage() -> &'static str {
    "usage: afterburner debug deploy reconcile-verification-bundle-rollback --rollback PATH --current-rollback PATH [--out PATH]"
}
