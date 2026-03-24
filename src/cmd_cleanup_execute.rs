use std::fs;
use std::path::{Component, Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const ARTIFACT_CLEANUP_EXECUTION_RECEIPT_PATH: &str =
    "artifacts/deploy/artifact_cleanup_execution_receipt.json";

#[derive(Debug)]
enum CleanupExecuteError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for CleanupExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CleanupExecuteError {}

impl From<std::io::Error> for CleanupExecuteError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for CleanupExecuteError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize cleanup execution receipt json: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    dry_run_receipt_path: PathBuf,
    artifacts_root: PathBuf,
    executed_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), CleanupExecuteError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let dry_run_receipt = load_json_object(&args.dry_run_receipt_path)?;
    let policy_profile = dry_run_receipt
        .get("policy_profile")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CleanupExecuteError::Parse(format!(
                "cleanup dry-run receipt `{}` must contain `policy_profile`",
                args.dry_run_receipt_path.display()
            ))
        })?;
    let planned_removals = dry_run_receipt
        .get("planned_removals")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CleanupExecuteError::Parse(format!(
                "cleanup dry-run receipt `{}` must contain `planned_removals` array",
                args.dry_run_receipt_path.display()
            ))
        })?;
    let inventory_path = dry_run_receipt
        .get("inventory_path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CleanupExecuteError::Parse(format!(
                "cleanup dry-run receipt `{}` must contain `inventory_path`",
                args.dry_run_receipt_path.display()
            ))
        })?;
    let policy_path = dry_run_receipt
        .get("policy_path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CleanupExecuteError::Parse(format!(
                "cleanup dry-run receipt `{}` must contain `policy_path`",
                args.dry_run_receipt_path.display()
            ))
        })?;
    let observed_at_unix_ms = dry_run_receipt
        .get("generated_at_unix_ms")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            CleanupExecuteError::Parse(format!(
                "cleanup dry-run receipt `{}` must contain `generated_at_unix_ms`",
                args.dry_run_receipt_path.display()
            ))
        })?;

    let mut removed_paths = Vec::new();
    let mut skipped_paths = Vec::new();

    for entry in planned_removals {
        let relative = entry.as_str().ok_or_else(|| {
            CleanupExecuteError::Parse(format!(
                "cleanup dry-run receipt `{}` has non-string planned removal",
                args.dry_run_receipt_path.display()
            ))
        })?;
        let absolute = resolve_cleanup_path(args.artifacts_root.as_path(), relative)?;
        if !absolute.exists() {
            skipped_paths.push(relative.to_string());
            continue;
        }

        let metadata = fs::symlink_metadata(&absolute)?;
        if metadata.is_dir() {
            fs::remove_dir_all(&absolute)?;
        } else {
            fs::remove_file(&absolute)?;
        }
        removed_paths.push(relative.to_string());
    }

    let receipt = json!({
        "schema_version": "1",
        "dry_run_receipt_path": args.dry_run_receipt_path.display().to_string(),
        "policy_profile": policy_profile,
        "evidence_sources": [
            {
                "artifact_path": args.dry_run_receipt_path.display().to_string(),
                "artifact_role": "cleanup_dry_run_receipt",
                "observed_at_unix_ms": observed_at_unix_ms
            },
            {
                "artifact_path": inventory_path,
                "artifact_role": "cleanup_inventory",
                "observed_at_unix_ms": observed_at_unix_ms
            },
            {
                "artifact_path": policy_path,
                "artifact_role": "cleanup_policy",
                "observed_at_unix_ms": observed_at_unix_ms
            }
        ],
        "removed_paths": removed_paths,
        "skipped_paths": skipped_paths,
        "executed_at_unix_ms": args.executed_at_unix_ms
    });

    write_json_value(&args.out_path, &receipt)?;
    emit_json_artifact_written(
        "ops_cli",
        "artifact_cleanup_execution_receipt_written",
        "receipt_path",
        &args.out_path,
        "receipt",
        receipt,
    );
    Ok(())
}

fn resolve_cleanup_path(
    artifacts_root: &Path,
    relative: &str,
) -> Result<PathBuf, CleanupExecuteError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        return Err(CleanupExecuteError::Parse(format!(
            "planned removal `{relative}` must be relative to the artifacts root"
        )));
    }
    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(CleanupExecuteError::Parse(format!(
            "planned removal `{relative}` must not escape the artifacts root"
        )));
    }
    Ok(artifacts_root.join(relative_path))
}

fn load_json_object(path: &PathBuf) -> Result<serde_json::Map<String, Value>, CleanupExecuteError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        CleanupExecuteError::Parse(format!("parse json at {}: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        CleanupExecuteError::Parse(format!("json at {} must be an object", path.display()))
    })
}

fn parse_args<I>(args: I) -> Result<Args, CleanupExecuteError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut dry_run_receipt_path = None::<PathBuf>;
    let mut artifacts_root = PathBuf::from("artifacts");
    let mut executed_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(ARTIFACT_CLEANUP_EXECUTION_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dry-run-receipt" => {
                dry_run_receipt_path = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--dry-run-receipt",
                )?))
            }
            "--artifacts-root" => {
                artifacts_root =
                    PathBuf::from(parse_value::<String, _>(&mut args, "--artifacts-root")?)
            }
            "--executed-at-unix-ms" => {
                executed_at_unix_ms = Some(parse_value(&mut args, "--executed-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--dry-run-receipt=") => {
                dry_run_receipt_path = Some(PathBuf::from(
                    arg.trim_start_matches("--dry-run-receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--artifacts-root=") => {
                artifacts_root =
                    PathBuf::from(arg.trim_start_matches("--artifacts-root=").to_string())
            }
            _ if arg.starts_with("--executed-at-unix-ms=") => {
                executed_at_unix_ms = Some(
                    arg.trim_start_matches("--executed-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            CleanupExecuteError::InvalidArg(format!(
                                "invalid value for --executed-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(CleanupExecuteError::InvalidArg(format!(
                    "unknown argument for cleanup execute: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        dry_run_receipt_path: dry_run_receipt_path.ok_or_else(|| {
            CleanupExecuteError::InvalidArg(format!(
                "missing value for --dry-run-receipt\n{}",
                usage()
            ))
        })?,
        artifacts_root,
        executed_at_unix_ms: executed_at_unix_ms.ok_or_else(|| {
            CleanupExecuteError::InvalidArg(format!(
                "missing value for --executed-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, CleanupExecuteError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        CleanupExecuteError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        CleanupExecuteError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner cleanup execute --dry-run-receipt PATH [--artifacts-root PATH] --executed-at-unix-ms N [--out PATH]"
}
