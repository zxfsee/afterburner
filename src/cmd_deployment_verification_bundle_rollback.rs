use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const BUNDLE_OUT_PATH: &str = "artifacts/deploy/deployment_verification_evidence_bundle.json";
const ROLLBACK_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_rollback.json";

#[derive(Debug)]
enum DeploymentVerificationBundleRollbackError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleRollbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleRollbackError {}

impl From<std::io::Error> for DeploymentVerificationBundleRollbackError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    current_bundle: PathBuf,
    restored_bundle: PathBuf,
    rolled_back_at_unix_ms: u64,
    out_bundle: PathBuf,
    out_record: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleRollbackError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let current_bundle = load_json_object(
        args.current_bundle.as_path(),
        "deployment verification evidence bundle",
    )?;
    let restored_bundle = load_json_object(
        args.restored_bundle.as_path(),
        "deployment verification evidence bundle",
    )?;

    let current_profile = read_string(
        &current_bundle,
        "profile_name",
        args.current_bundle.as_path(),
        "deployment verification evidence bundle",
    )?;
    let restored_profile = read_string(
        &restored_bundle,
        "profile_name",
        args.restored_bundle.as_path(),
        "deployment verification evidence bundle",
    )?;
    if current_profile != restored_profile {
        return Err(DeploymentVerificationBundleRollbackError::Parse(format!(
            "current bundle profile_name `{current_profile}` does not match restored bundle profile_name `{restored_profile}`"
        )));
    }

    let restored_bundle_value = Value::Object(restored_bundle.clone());
    write_json(
        args.out_bundle.as_path(),
        &restored_bundle_value,
        "restored bundle",
    )?;

    let rollback = json!({
        "schema_version": "1",
        "current_bundle_path": args.current_bundle.display().to_string(),
        "restored_bundle_path": args.restored_bundle.display().to_string(),
        "previous_receipt_path": read_string(&current_bundle, "receipt_path", args.current_bundle.as_path(), "deployment verification evidence bundle")?,
        "restored_receipt_path": read_string(&restored_bundle, "receipt_path", args.restored_bundle.as_path(), "deployment verification evidence bundle")?,
        "previous_artifact_version": read_string(&current_bundle, "artifact_version", args.current_bundle.as_path(), "deployment verification evidence bundle")?,
        "restored_artifact_version": read_string(&restored_bundle, "artifact_version", args.restored_bundle.as_path(), "deployment verification evidence bundle")?,
        "profile_name": restored_profile,
        "previous_verification_status": read_string(&current_bundle, "verification_status", args.current_bundle.as_path(), "deployment verification evidence bundle")?,
        "restored_verification_status": read_string(&restored_bundle, "verification_status", args.restored_bundle.as_path(), "deployment verification evidence bundle")?,
        "previous_evidence_count": read_u64(&current_bundle, "evidence_count", args.current_bundle.as_path(), "deployment verification evidence bundle")?,
        "restored_evidence_count": read_u64(&restored_bundle, "evidence_count", args.restored_bundle.as_path(), "deployment verification evidence bundle")?,
        "rolled_back_at_unix_ms": args.rolled_back_at_unix_ms,
    });
    write_json(args.out_record.as_path(), &rollback, "rollback")?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_verification_evidence_bundle_rolled_back",
        json!({
            "bundle_path": args.out_bundle.display().to_string(),
            "rollback_path": args.out_record.display().to_string(),
            "bundle": restored_bundle_value,
            "rollback": rollback,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationBundleRollbackError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut current_bundle = None::<PathBuf>;
    let mut restored_bundle = None::<PathBuf>;
    let mut rolled_back_at_unix_ms = None::<u64>;
    let mut out_bundle = PathBuf::from(BUNDLE_OUT_PATH);
    let mut out_record = PathBuf::from(ROLLBACK_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--current-bundle" => {
                current_bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-bundle",
                )?))
            }
            "--restored-bundle" => {
                restored_bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--restored-bundle",
                )?))
            }
            "--rolled-back-at-unix-ms" => {
                rolled_back_at_unix_ms = Some(parse_value(&mut args, "--rolled-back-at-unix-ms")?)
            }
            "--out-bundle" => {
                out_bundle = PathBuf::from(parse_value::<String, _>(&mut args, "--out-bundle")?)
            }
            "--out-record" => {
                out_record = PathBuf::from(parse_value::<String, _>(&mut args, "--out-record")?)
            }
            _ if arg.starts_with("--current-bundle=") => {
                current_bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--current-bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--restored-bundle=") => {
                restored_bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--restored-bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--rolled-back-at-unix-ms=") => {
                rolled_back_at_unix_ms = Some(
                    arg.trim_start_matches("--rolled-back-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DeploymentVerificationBundleRollbackError::InvalidArg(format!(
                                "invalid value for --rolled-back-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out-bundle=") => {
                out_bundle = PathBuf::from(arg.trim_start_matches("--out-bundle=").to_string())
            }
            _ if arg.starts_with("--out-record=") => {
                out_record = PathBuf::from(arg.trim_start_matches("--out-record=").to_string())
            }
            _ => {
                return Err(DeploymentVerificationBundleRollbackError::InvalidArg(
                    format!(
                        "unknown argument for deploy rollback-verification-bundle: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        current_bundle: current_bundle.ok_or_else(|| {
            DeploymentVerificationBundleRollbackError::InvalidArg(format!(
                "missing value for --current-bundle\n{}",
                usage()
            ))
        })?,
        restored_bundle: restored_bundle.ok_or_else(|| {
            DeploymentVerificationBundleRollbackError::InvalidArg(format!(
                "missing value for --restored-bundle\n{}",
                usage()
            ))
        })?,
        rolled_back_at_unix_ms: rolled_back_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationBundleRollbackError::InvalidArg(format!(
                "missing value for --rolled-back-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_bundle,
        out_record,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationBundleRollbackError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleRollbackError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleRollbackError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentVerificationBundleRollbackError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationBundleRollbackError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentVerificationBundleRollbackError::Parse(format!(
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
) -> Result<String, DeploymentVerificationBundleRollbackError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentVerificationBundleRollbackError::Parse(format!(
                "{kind} `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<u64, DeploymentVerificationBundleRollbackError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        DeploymentVerificationBundleRollbackError::Parse(format!(
            "{kind} `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn write_json(
    path: &Path,
    value: &Value,
    kind: &str,
) -> Result<(), DeploymentVerificationBundleRollbackError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|err| {
        DeploymentVerificationBundleRollbackError::Parse(format!("serialize {kind} json: {err}"))
    })?;
    fs::write(path, text)?;
    Ok(())
}

fn usage() -> &'static str {
    "usage: afterburner rollback verification-bundle --current-bundle PATH --restored-bundle PATH --rolled-back-at-unix-ms N [--out-bundle PATH] [--out-record PATH]"
}
