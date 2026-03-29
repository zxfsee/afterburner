use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const DEPLOYMENT_VERIFICATION_BUNDLE_ROLLBACK_SUPERSESSION_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.json";

#[derive(Debug)]
enum DeploymentVerificationBundleRollbackSupersessionError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleRollbackSupersessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleRollbackSupersessionError {}

impl From<std::io::Error> for DeploymentVerificationBundleRollbackSupersessionError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    previous_rollback: PathBuf,
    next_rollback: PathBuf,
    superseded_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleRollbackSupersessionError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let previous = load_json_object(
        args.previous_rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;
    let next = load_json_object(
        args.next_rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;

    let previous_profile = read_string(
        &previous,
        "profile_name",
        args.previous_rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;
    let next_profile = read_string(
        &next,
        "profile_name",
        args.next_rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;
    if previous_profile != next_profile {
        return Err(
            DeploymentVerificationBundleRollbackSupersessionError::Parse(format!(
                "rollback records must share one profile_name, found `{previous_profile}` and `{next_profile}`"
            )),
        );
    }

    let previous_restored_artifact_version = read_string(
        &previous,
        "restored_artifact_version",
        args.previous_rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;
    let next_restored_artifact_version = read_string(
        &next,
        "restored_artifact_version",
        args.next_rollback.as_path(),
        "deployment verification evidence bundle rollback",
    )?;
    if previous_restored_artifact_version == next_restored_artifact_version {
        return Err(
            DeploymentVerificationBundleRollbackSupersessionError::Parse(format!(
                "next rollback `{}` must target a different restored_artifact_version than previous rollback `{}`",
                args.next_rollback.display(),
                args.previous_rollback.display()
            )),
        );
    }

    let supersession = json!({
        "schema_version": "1",
        "previous_rollback_path": args.previous_rollback.display().to_string(),
        "next_rollback_path": args.next_rollback.display().to_string(),
        "previous_restored_bundle_path": read_string(&previous, "restored_bundle_path", args.previous_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "next_restored_bundle_path": read_string(&next, "restored_bundle_path", args.next_rollback.as_path(), "deployment verification evidence bundle rollback")?,
        "previous_restored_artifact_version": previous_restored_artifact_version,
        "next_restored_artifact_version": next_restored_artifact_version,
        "profile_name": next_profile,
        "superseded_at_unix_ms": args.superseded_at_unix_ms,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&supersession).map_err(|err| {
        DeploymentVerificationBundleRollbackSupersessionError::Parse(format!(
            "serialize deployment verification bundle rollback supersession json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_verification_evidence_bundle_rollback_superseded",
        json!({
            "supersession_path": args.out_path.display().to_string(),
            "supersession": supersession,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationBundleRollbackSupersessionError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut previous_rollback = None::<PathBuf>;
    let mut next_rollback = None::<PathBuf>;
    let mut superseded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEPLOYMENT_VERIFICATION_BUNDLE_ROLLBACK_SUPERSESSION_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--previous-rollback" => {
                let value: String = parse_value(&mut args, "--previous-rollback")?;
                previous_rollback = Some(PathBuf::from(value));
            }
            "--next-rollback" => {
                let value: String = parse_value(&mut args, "--next-rollback")?;
                next_rollback = Some(PathBuf::from(value));
            }
            "--superseded-at-unix-ms" => {
                superseded_at_unix_ms = Some(parse_value(&mut args, "--superseded-at-unix-ms")?)
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--previous-rollback=") => {
                previous_rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--previous-rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--next-rollback=") => {
                next_rollback = Some(PathBuf::from(
                    arg.trim_start_matches("--next-rollback=").to_string(),
                ))
            }
            _ if arg.starts_with("--superseded-at-unix-ms=") => {
                superseded_at_unix_ms = Some(
                    arg.trim_start_matches("--superseded-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DeploymentVerificationBundleRollbackSupersessionError::InvalidArg(
                                format!("invalid value for --superseded-at-unix-ms\n{}", usage()),
                            )
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationBundleRollbackSupersessionError::InvalidArg(format!(
                        "unknown argument for deploy supersede-verification-bundle-rollback: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        previous_rollback: previous_rollback.ok_or_else(|| {
            DeploymentVerificationBundleRollbackSupersessionError::InvalidArg(format!(
                "missing value for --previous-rollback\n{}",
                usage()
            ))
        })?,
        next_rollback: next_rollback.ok_or_else(|| {
            DeploymentVerificationBundleRollbackSupersessionError::InvalidArg(format!(
                "missing value for --next-rollback\n{}",
                usage()
            ))
        })?,
        superseded_at_unix_ms: superseded_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationBundleRollbackSupersessionError::InvalidArg(format!(
                "missing value for --superseded-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationBundleRollbackSupersessionError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleRollbackSupersessionError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleRollbackSupersessionError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentVerificationBundleRollbackSupersessionError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationBundleRollbackSupersessionError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentVerificationBundleRollbackSupersessionError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<String, DeploymentVerificationBundleRollbackSupersessionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentVerificationBundleRollbackSupersessionError::Parse(format!(
                "{kind} `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner deploy supersede-verification-bundle-rollback --previous-rollback PATH --next-rollback PATH --superseded-at-unix-ms N [--out PATH]"
}
