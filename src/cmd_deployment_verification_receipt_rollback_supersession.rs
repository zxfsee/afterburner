use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::json;

const DEPLOYMENT_VERIFICATION_RECEIPT_ROLLBACK_SUPERSESSION_PATH: &str =
    "artifacts/deploy/deployment_verification_receipt_rollback_supersession.json";

#[derive(Debug)]
enum DeploymentVerificationReceiptRollbackSupersessionError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationReceiptRollbackSupersessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationReceiptRollbackSupersessionError {}

impl From<std::io::Error> for DeploymentVerificationReceiptRollbackSupersessionError {
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationReceiptRollbackSupersessionError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let previous = load_json_object(
        args.previous_rollback.as_path(),
        "deployment verification receipt rollback",
    )?;
    let next = load_json_object(
        args.next_rollback.as_path(),
        "deployment verification receipt rollback",
    )?;

    let previous_profile = read_string(
        &previous,
        "profile_name",
        args.previous_rollback.as_path(),
        "deployment verification receipt rollback",
    )?;
    let next_profile = read_string(
        &next,
        "profile_name",
        args.next_rollback.as_path(),
        "deployment verification receipt rollback",
    )?;
    if previous_profile != next_profile {
        return Err(
            DeploymentVerificationReceiptRollbackSupersessionError::Parse(format!(
                "rollback records must share one profile_name, found `{previous_profile}` and `{next_profile}`"
            )),
        );
    }

    let previous_restored_artifact_version = read_string(
        &previous,
        "restored_artifact_version",
        args.previous_rollback.as_path(),
        "deployment verification receipt rollback",
    )?;
    let next_restored_artifact_version = read_string(
        &next,
        "restored_artifact_version",
        args.next_rollback.as_path(),
        "deployment verification receipt rollback",
    )?;
    if previous_restored_artifact_version == next_restored_artifact_version {
        return Err(
            DeploymentVerificationReceiptRollbackSupersessionError::Parse(format!(
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
        "previous_restored_receipt_path": read_string(&previous, "restored_receipt_path", args.previous_rollback.as_path(), "deployment verification receipt rollback")?,
        "next_restored_receipt_path": read_string(&next, "restored_receipt_path", args.next_rollback.as_path(), "deployment verification receipt rollback")?,
        "previous_restored_artifact_version": previous_restored_artifact_version,
        "next_restored_artifact_version": next_restored_artifact_version,
        "profile_name": next_profile,
        "superseded_at_unix_ms": args.superseded_at_unix_ms,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&supersession).map_err(|err| {
        DeploymentVerificationReceiptRollbackSupersessionError::Parse(format!(
            "serialize deployment verification receipt rollback supersession json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_verification_receipt_rollback_superseded",
        json!({
            "supersession_path": args.out_path.display().to_string(),
            "supersession": supersession,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationReceiptRollbackSupersessionError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut previous_rollback = None::<PathBuf>;
    let mut next_rollback = None::<PathBuf>;
    let mut superseded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEPLOYMENT_VERIFICATION_RECEIPT_ROLLBACK_SUPERSESSION_PATH);

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
                            DeploymentVerificationReceiptRollbackSupersessionError::InvalidArg(
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
                    DeploymentVerificationReceiptRollbackSupersessionError::InvalidArg(format!(
                        "unknown argument for deploy supersede-verification-receipt-rollback: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        previous_rollback: previous_rollback.ok_or_else(|| {
            DeploymentVerificationReceiptRollbackSupersessionError::InvalidArg(format!(
                "missing value for --previous-rollback\n{}",
                usage()
            ))
        })?,
        next_rollback: next_rollback.ok_or_else(|| {
            DeploymentVerificationReceiptRollbackSupersessionError::InvalidArg(format!(
                "missing value for --next-rollback\n{}",
                usage()
            ))
        })?,
        superseded_at_unix_ms: superseded_at_unix_ms.ok_or_else(|| {
            DeploymentVerificationReceiptRollbackSupersessionError::InvalidArg(format!(
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
) -> Result<T, DeploymentVerificationReceiptRollbackSupersessionError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationReceiptRollbackSupersessionError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationReceiptRollbackSupersessionError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(
    DeploymentVerificationReceiptRollbackSupersessionError
);

fn usage() -> &'static str {
    "usage: afterburner debug deploy supersede-verification-receipt-rollback --previous-rollback PATH --next-rollback PATH --superseded-at-unix-ms N [--out PATH]"
}
