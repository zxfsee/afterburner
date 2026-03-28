use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, load_json_object, read_string, read_u64, reconciliation_status,
    write_emitted_json,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_handoff_reconciliation.json";

#[derive(Debug)]
enum DeploymentVerificationHandoffReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationHandoffReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationHandoffReconcileError {}

impl From<std::io::Error> for DeploymentVerificationHandoffReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for DeploymentVerificationHandoffReconcileError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize deployment verification evidence handoff reconciliation json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError> for DeploymentVerificationHandoffReconcileError {
    fn from(value: ReconciliationError) -> Self {
        match value {
            ReconciliationError::Io(err) => Self::Io(err),
            ReconciliationError::Parse(msg) => Self::Parse(msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    bundle: PathBuf,
    handoff: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationHandoffReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "deployment verification evidence bundle",
    )?;
    let handoff = load_json_object(
        args.handoff.as_path(),
        "deployment verification evidence handoff",
    )?;

    let desired = json!({
        "artifact_version": read_string(&bundle, "artifact_version", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "profile_name": read_string(&bundle, "profile_name", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "verification_status": read_string(&bundle, "verification_status", args.bundle.as_path(), "deployment verification evidence bundle")?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path(), "deployment verification evidence bundle")?,
    });
    let current = json!({
        "bundle_path": read_string(&handoff, "bundle_path", args.handoff.as_path(), "deployment verification evidence handoff")?,
        "artifact_version": read_string(&handoff, "artifact_version", args.handoff.as_path(), "deployment verification evidence handoff")?,
        "profile_name": read_string(&handoff, "profile_name", args.handoff.as_path(), "deployment verification evidence handoff")?,
        "verification_status": read_string(&handoff, "verification_status", args.handoff.as_path(), "deployment verification evidence handoff")?,
        "evidence_count": read_u64(&handoff, "evidence_count", args.handoff.as_path(), "deployment verification evidence handoff")?,
    });
    let current_comparable = json!({
        "artifact_version": current["artifact_version"].clone(),
        "profile_name": current["profile_name"].clone(),
        "verification_status": current["verification_status"].clone(),
        "evidence_count": current["evidence_count"].clone(),
    });
    let reconciliation_status = reconciliation_status(&desired, &current_comparable);

    let reconciliation = json!({
        "schema_version": "1",
        "bundle_path": args.bundle.display().to_string(),
        "handoff_path": args.handoff.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_emitted_json(
        "deploy_cli",
        "deployment_verification_evidence_handoff_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    )?;
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationHandoffReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut bundle = None::<PathBuf>;
    let mut handoff = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bundle" => {
                bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--bundle",
                )?))
            }
            "--handoff" => {
                handoff = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--handoff",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--bundle=") => {
                bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--handoff=") => {
                handoff = Some(PathBuf::from(
                    arg.trim_start_matches("--handoff=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentVerificationHandoffReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy reconcile-verification-handoff: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        bundle: bundle.ok_or_else(|| {
            DeploymentVerificationHandoffReconcileError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        handoff: handoff.ok_or_else(|| {
            DeploymentVerificationHandoffReconcileError::InvalidArg(format!(
                "missing value for --handoff\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationHandoffReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationHandoffReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationHandoffReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy reconcile-verification-handoff --bundle PATH --handoff PATH [--out PATH]"
}
