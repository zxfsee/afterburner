use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const POINTER_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer.json";
const ROLLBACK_OUT_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer_rollback.json";

#[derive(Debug)]
enum DeploymentVerificationBundleLocatorRollbackError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationBundleLocatorRollbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationBundleLocatorRollbackError {}

impl From<std::io::Error> for DeploymentVerificationBundleLocatorRollbackError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    current_pointer: PathBuf,
    restored_pointer: PathBuf,
    rolled_back_at_unix_ms: u64,
    out_pointer: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationBundleLocatorRollbackError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let current_pointer = load_json_object(
        args.current_pointer.as_path(),
        "deployment verification evidence bundle locator pointer",
    )?;
    let restored_pointer = load_json_object(
        args.restored_pointer.as_path(),
        "deployment verification evidence bundle locator pointer",
    )?;

    let current_profile = read_string(
        &current_pointer,
        "profile_name",
        args.current_pointer.as_path(),
        "deployment verification evidence bundle locator pointer",
    )?;
    let restored_profile = read_string(
        &restored_pointer,
        "profile_name",
        args.restored_pointer.as_path(),
        "deployment verification evidence bundle locator pointer",
    )?;
    if current_profile != restored_profile {
        return Err(DeploymentVerificationBundleLocatorRollbackError::Parse(
            format!(
                "current pointer profile_name `{current_profile}` does not match restored pointer profile_name `{restored_profile}`"
            ),
        ));
    }

    let restored_pointer_value = Value::Object(restored_pointer.clone());
    write_json(
        args.out_pointer.as_path(),
        &restored_pointer_value,
        "restored pointer",
    )?;

    let rollback = json!({
        "schema_version": "1",
        "current_pointer_path": args.current_pointer.display().to_string(),
        "restored_pointer_path": args.restored_pointer.display().to_string(),
        "previous_locator_path": read_string(&current_pointer, "locator_path", args.current_pointer.as_path(), "deployment verification evidence bundle locator pointer")?,
        "restored_locator_path": read_string(&restored_pointer, "locator_path", args.restored_pointer.as_path(), "deployment verification evidence bundle locator pointer")?,
        "previous_bundle_path": read_string(&current_pointer, "bundle_path", args.current_pointer.as_path(), "deployment verification evidence bundle locator pointer")?,
        "restored_bundle_path": read_string(&restored_pointer, "bundle_path", args.restored_pointer.as_path(), "deployment verification evidence bundle locator pointer")?,
        "previous_artifact_version": read_string(&current_pointer, "artifact_version", args.current_pointer.as_path(), "deployment verification evidence bundle locator pointer")?,
        "restored_artifact_version": read_string(&restored_pointer, "artifact_version", args.restored_pointer.as_path(), "deployment verification evidence bundle locator pointer")?,
        "profile_name": restored_profile,
        "rolled_back_at_unix_ms": args.rolled_back_at_unix_ms,
    });
    write_json(args.out_record.as_path(), &rollback, "rollback")?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_verification_evidence_bundle_locator_pointer_rolled_back",
        json!({
            "pointer_path": args.out_pointer.display().to_string(),
            "rollback_path": args.out_record.display().to_string(),
            "pointer": restored_pointer_value,
            "rollback": rollback,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationBundleLocatorRollbackError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut current_pointer = None::<PathBuf>;
    let mut restored_pointer = None::<PathBuf>;
    let mut rolled_back_at_unix_ms = None::<u64>;
    let mut out_pointer = PathBuf::from(POINTER_OUT_PATH);
    let mut out_record = PathBuf::from(ROLLBACK_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--current-pointer" => {
                current_pointer = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--current-pointer",
                )?))
            }
            "--restored-pointer" => {
                restored_pointer = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--restored-pointer",
                )?))
            }
            "--rolled-back-at-unix-ms" => {
                rolled_back_at_unix_ms = Some(parse_value(&mut args, "--rolled-back-at-unix-ms")?)
            }
            "--out-pointer" => {
                out_pointer = PathBuf::from(parse_value::<String, _>(&mut args, "--out-pointer")?)
            }
            "--out-record" => {
                out_record = PathBuf::from(parse_value::<String, _>(&mut args, "--out-record")?)
            }
            _ if arg.starts_with("--current-pointer=") => {
                current_pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--current-pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--restored-pointer=") => {
                restored_pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--restored-pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--rolled-back-at-unix-ms=") => {
                rolled_back_at_unix_ms = Some(
                    arg.trim_start_matches("--rolled-back-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DeploymentVerificationBundleLocatorRollbackError::InvalidArg(format!(
                                "invalid value for --rolled-back-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out-pointer=") => {
                out_pointer = PathBuf::from(arg.trim_start_matches("--out-pointer=").to_string())
            }
            _ if arg.starts_with("--out-record=") => {
                out_record = PathBuf::from(arg.trim_start_matches("--out-record=").to_string())
            }
            _ => {
                return Err(
                    DeploymentVerificationBundleLocatorRollbackError::InvalidArg(format!(
                        "unknown argument for deploy rollback-verification-bundle-locator: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        current_pointer: current_pointer.ok_or_else(|| {
            DeploymentVerificationBundleLocatorRollbackError::InvalidArg(format!(
                "missing value for --current-pointer\n{}",
                usage()
            ))
        })?,
        restored_pointer: restored_pointer.ok_or_else(|| {
            DeploymentVerificationBundleLocatorRollbackError::InvalidArg(format!(
                "missing value for --restored-pointer\n{}",
                usage()
            ))
        })?,
        rolled_back_at_unix_ms: rolled_back_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationBundleLocatorRollbackError::InvalidArg(format!(
                "missing value for --rolled-back-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_pointer,
        out_record,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentVerificationBundleLocatorRollbackError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationBundleLocatorRollbackError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationBundleLocatorRollbackError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(DeploymentVerificationBundleLocatorRollbackError);

fn usage() -> &'static str {
    "usage: afterburner rollback verification-bundle-locator --current-pointer PATH --restored-pointer PATH --rolled-back-at-unix-ms N [--out-pointer PATH] [--out-record PATH]"
}
