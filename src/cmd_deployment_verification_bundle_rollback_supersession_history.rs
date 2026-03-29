use std::path::PathBuf;

use afterburner::command_reconciliation::{
    ReconciliationError, append_history_entry, load_json_object, load_or_init_history, read_string,
    read_u64, write_emitted_json,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession_history.json";

#[derive(Debug)]
enum DeploymentVerificationBundleRollbackSupersessionHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleRollbackSupersessionHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleRollbackSupersessionHistoryError {}

impl From<afterburner::command_artifacts::JsonArtifactError>
    for DeploymentVerificationBundleRollbackSupersessionHistoryError
{
    fn from(value: afterburner::command_artifacts::JsonArtifactError) -> Self {
        match value {
            afterburner::command_artifacts::JsonArtifactError::Io(err) => Self::Io(err),
            afterburner::command_artifacts::JsonArtifactError::Serialize(err) => {
                Self::Parse(format!(
                    "serialize deployment verification bundle rollback supersession history json: {err}"
                ))
            }
        }
    }
}

impl From<ReconciliationError> for DeploymentVerificationBundleRollbackSupersessionHistoryError {
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleRollbackSupersessionHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let supersession = load_json_object(
        args.supersession.as_path(),
        "deployment verification bundle rollback supersession",
    )?;

    let mut history = load_or_init_history(args.out_path.as_path())?;
    let entry = json!({
        "event": args.event,
        "supersession_path": args.supersession.display().to_string(),
        "previous_rollback_path": read_string(&supersession, "previous_rollback_path", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "next_rollback_path": read_string(&supersession, "next_rollback_path", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "previous_restored_bundle_path": read_string(&supersession, "previous_restored_bundle_path", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "next_restored_bundle_path": read_string(&supersession, "next_restored_bundle_path", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "previous_restored_artifact_version": read_string(&supersession, "previous_restored_artifact_version", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "next_restored_artifact_version": read_string(&supersession, "next_restored_artifact_version", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "profile_name": read_string(&supersession, "profile_name", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "superseded_at_unix_ms": read_u64(&supersession, "superseded_at_unix_ms", args.supersession.as_path(), "deployment verification bundle rollback supersession")?,
        "recorded_at_unix_ms": args.recorded_at_unix_ms,
    });
    append_history_entry(&mut history, entry);

    write_emitted_json(
        "deploy_cli",
        "deployment_verification_evidence_bundle_rollback_supersession_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    )?;
    Ok(())
}

fn parse_args<I>(
    args: I,
) -> Result<Args, DeploymentVerificationBundleRollbackSupersessionHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut supersession = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--supersession" => {
                supersession = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--supersession",
                )?))
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--supersession=") => {
                supersession = Some(PathBuf::from(
                    arg.trim_start_matches("--supersession=").to_string(),
                ))
            }
            _ if arg.starts_with("--event=") => {
                event = Some(arg.trim_start_matches("--event=").to_string())
            }
            _ if arg.starts_with("--recorded-at-unix-ms=") => recorded_at_unix_ms = Some(
                arg.trim_start_matches("--recorded-at-unix-ms=")
                    .parse::<u64>()
                    .map_err(|_| {
                        DeploymentVerificationBundleRollbackSupersessionHistoryError::InvalidArg(
                            format!("invalid value for --recorded-at-unix-ms\n{}", usage()),
                        )
                    })?,
            ),
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationBundleRollbackSupersessionHistoryError::InvalidArg(
                        format!(
                            "unknown argument for deploy record-verification-bundle-rollback-supersession-history: {arg}\n{}",
                            usage()
                        ),
                    ),
                );
            }
        }
    }

    Ok(Args {
        supersession: supersession.ok_or_else(|| {
            DeploymentVerificationBundleRollbackSupersessionHistoryError::InvalidArg(format!(
                "missing value for --supersession\n{}",
                usage()
            ))
        })?,
        event: event.ok_or_else(|| {
            DeploymentVerificationBundleRollbackSupersessionHistoryError::InvalidArg(format!(
                "missing value for --event\n{}",
                usage()
            ))
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationBundleRollbackSupersessionHistoryError::InvalidArg(format!(
                "missing value for --recorded-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationBundleRollbackSupersessionHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleRollbackSupersessionHistoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleRollbackSupersessionHistoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy record-verification-bundle-rollback-supersession-history --supersession PATH --event NAME --recorded-at-unix-ms MS [--out PATH]"
}
