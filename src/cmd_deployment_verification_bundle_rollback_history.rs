use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_rollback_history.json";

#[derive(Debug)]
enum DeploymentVerificationBundleRollbackHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleRollbackHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleRollbackHistoryError {}

impl From<std::io::Error> for DeploymentVerificationBundleRollbackHistoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentVerificationBundleRollbackHistoryError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment verification bundle rollback history json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    rollback: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleRollbackHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let rollback = load_json_object(
        args.rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;

    let mut history = if args.out_path.exists() {
        load_history(args.out_path.as_path())?
    } else {
        json!({
            "schema_version": "1",
            "entries": []
        })
    };

    let entry = json!({
        "event": args.event,
        "rollback_path": args.rollback.display().to_string(),
        "current_bundle_path": read_string(&rollback, "current_bundle_path", args.rollback.as_path())?,
        "restored_bundle_path": read_string(&rollback, "restored_bundle_path", args.rollback.as_path())?,
        "previous_receipt_path": read_string(&rollback, "previous_receipt_path", args.rollback.as_path())?,
        "restored_receipt_path": read_string(&rollback, "restored_receipt_path", args.rollback.as_path())?,
        "previous_artifact_version": read_string(&rollback, "previous_artifact_version", args.rollback.as_path())?,
        "restored_artifact_version": read_string(&rollback, "restored_artifact_version", args.rollback.as_path())?,
        "profile_name": read_string(&rollback, "profile_name", args.rollback.as_path())?,
        "previous_verification_status": read_string(&rollback, "previous_verification_status", args.rollback.as_path())?,
        "restored_verification_status": read_string(&rollback, "restored_verification_status", args.rollback.as_path())?,
        "previous_evidence_count": read_u64(&rollback, "previous_evidence_count", args.rollback.as_path())?,
        "restored_evidence_count": read_u64(&rollback, "restored_evidence_count", args.rollback.as_path())?,
        "rolled_back_at_unix_ms": read_u64(&rollback, "rolled_back_at_unix_ms", args.rollback.as_path())?,
        "recorded_at_unix_ms": args.recorded_at_unix_ms,
    });
    history_entries_mut(&mut history)?.push(entry);

    write_json_value(&args.out_path, &history)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_verification_evidence_bundle_rollback_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationBundleRollbackHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut rollback = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rollback" => {
                rollback = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--rollback",
                )?))
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--rollback=") => {
                rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--rollback=").to_string(),
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
                            DeploymentVerificationBundleRollbackHistoryError::InvalidArg(format!(
                                "invalid value for --recorded-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationBundleRollbackHistoryError::InvalidArg(format!(
                        "unknown argument for deploy record-verification-bundle-rollback-history: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        rollback: rollback.ok_or_else(|| {
            DeploymentVerificationBundleRollbackHistoryError::InvalidArg(format!(
                "missing value for --rollback\n{}",
                usage()
            ))
        })?,
        event: event.ok_or_else(|| {
            DeploymentVerificationBundleRollbackHistoryError::InvalidArg(format!(
                "missing value for --event\n{}",
                usage()
            ))
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationBundleRollbackHistoryError::InvalidArg(format!(
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
) -> Result<T, DeploymentVerificationBundleRollbackHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleRollbackHistoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleRollbackHistoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentVerificationBundleRollbackHistoryError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationBundleRollbackHistoryError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentVerificationBundleRollbackHistoryError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn load_history(path: &Path) -> Result<Value, DeploymentVerificationBundleRollbackHistoryError> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationBundleRollbackHistoryError::Parse(format!(
            "parse deployment verification bundle rollback history `{}`: {err}",
            path.display()
        ))
    })
}

fn history_entries_mut(
    history: &mut Value,
) -> Result<&mut Vec<Value>, DeploymentVerificationBundleRollbackHistoryError> {
    history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            DeploymentVerificationBundleRollbackHistoryError::Parse(
                "deployment verification bundle rollback history must contain an `entries` array"
                    .to_string(),
            )
        })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentVerificationBundleRollbackHistoryError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentVerificationBundleRollbackHistoryError::Parse(format!(
                "deployment verification bundle rollback `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, DeploymentVerificationBundleRollbackHistoryError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        DeploymentVerificationBundleRollbackHistoryError::Parse(format!(
            "deployment verification bundle rollback `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy record-verification-bundle-rollback-history --rollback PATH --event EVENT --recorded-at-unix-ms UNIX_MS [--out PATH]"
}
