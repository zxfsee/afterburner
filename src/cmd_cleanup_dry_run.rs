use std::fs;
use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const ARTIFACT_CLEANUP_DRY_RUN_RECEIPT_PATH: &str =
    "artifacts/deploy/artifact_cleanup_dry_run_receipt.json";

#[derive(Debug)]
enum CleanupDryRunError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for CleanupDryRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CleanupDryRunError {}

impl From<std::io::Error> for CleanupDryRunError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for CleanupDryRunError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize cleanup dry-run receipt json: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    inventory_path: PathBuf,
    policy_path: PathBuf,
    generated_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), CleanupDryRunError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let inventory = load_json_object(&args.inventory_path)?;
    let policy = load_json_object(&args.policy_path)?;

    let prune_candidates = inventory
        .get("prune_candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CleanupDryRunError::Parse(format!(
                "cleanup inventory `{}` must contain `prune_candidates` array",
                args.inventory_path.display()
            ))
        })?;
    let retained = inventory
        .get("retained")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CleanupDryRunError::Parse(format!(
                "cleanup inventory `{}` must contain `retained` array",
                args.inventory_path.display()
            ))
        })?;
    let policy_profile = policy
        .get("profile_name")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CleanupDryRunError::Parse(format!(
                "cleanup policy `{}` must contain `profile_name`",
                args.policy_path.display()
            ))
        })?;

    let planned_removals = prune_candidates
        .iter()
        .map(|candidate| {
            candidate
                .get("path")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| {
                    CleanupDryRunError::Parse(format!(
                        "cleanup inventory prune candidate in `{}` must contain string `path`",
                        args.inventory_path.display()
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let receipt = json!({
        "schema_version": "1",
        "inventory_path": args.inventory_path.display().to_string(),
        "policy_path": args.policy_path.display().to_string(),
        "policy_profile": policy_profile,
        "planned_removals": planned_removals,
        "retained_count": retained.len(),
        "prune_candidate_count": prune_candidates.len(),
        "generated_at_unix_ms": args.generated_at_unix_ms
    });

    write_json_value(&args.out_path, &receipt)?;
    emit_json_artifact_written(
        "ops_cli",
        "artifact_cleanup_dry_run_receipt_written",
        "receipt_path",
        &args.out_path,
        "receipt",
        receipt,
    );
    Ok(())
}

fn load_json_object(path: &PathBuf) -> Result<serde_json::Map<String, Value>, CleanupDryRunError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        CleanupDryRunError::Parse(format!("parse json at {}: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        CleanupDryRunError::Parse(format!("json at {} must be an object", path.display()))
    })
}

fn parse_args<I>(args: I) -> Result<Args, CleanupDryRunError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut inventory_path = None::<PathBuf>;
    let mut policy_path = None::<PathBuf>;
    let mut generated_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(ARTIFACT_CLEANUP_DRY_RUN_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--inventory" => {
                inventory_path = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--inventory",
                )?))
            }
            "--policy" => {
                policy_path = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--policy",
                )?))
            }
            "--generated-at-unix-ms" => {
                generated_at_unix_ms = Some(parse_value(&mut args, "--generated-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--inventory=") => {
                inventory_path = Some(PathBuf::from(
                    arg.trim_start_matches("--inventory=").to_string(),
                ))
            }
            _ if arg.starts_with("--policy=") => {
                policy_path = Some(PathBuf::from(
                    arg.trim_start_matches("--policy=").to_string(),
                ))
            }
            _ if arg.starts_with("--generated-at-unix-ms=") => {
                generated_at_unix_ms = Some(
                    arg.trim_start_matches("--generated-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            CleanupDryRunError::InvalidArg(format!(
                                "invalid value for --generated-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(CleanupDryRunError::InvalidArg(format!(
                    "unknown argument for cleanup dry-run: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        inventory_path: inventory_path.ok_or_else(|| {
            CleanupDryRunError::InvalidArg(format!("missing value for --inventory\n{}", usage()))
        })?,
        policy_path: policy_path.ok_or_else(|| {
            CleanupDryRunError::InvalidArg(format!("missing value for --policy\n{}", usage()))
        })?,
        generated_at_unix_ms: generated_at_unix_ms.ok_or_else(|| {
            CleanupDryRunError::InvalidArg(format!(
                "missing value for --generated-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, CleanupDryRunError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        CleanupDryRunError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        CleanupDryRunError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner cleanup dry-run --inventory PATH --policy PATH --generated-at-unix-ms N [--out PATH]"
}
