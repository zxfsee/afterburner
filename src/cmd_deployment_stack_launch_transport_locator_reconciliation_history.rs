use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_transport_locator_reconciliation_history.json";

#[derive(Debug)]
enum DeploymentStackLaunchTransportLocatorReconciliationHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchTransportLocatorReconciliationHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchTransportLocatorReconciliationHistoryError {}

impl From<std::io::Error> for DeploymentStackLaunchTransportLocatorReconciliationHistoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchTransportLocatorReconciliationHistoryError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch transport locator reconciliation history json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    reconciliation: PathBuf,
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

fn run_inner<I>(
    args: I,
) -> Result<(), DeploymentStackLaunchTransportLocatorReconciliationHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let reconciliation = load_json_object(
        args.reconciliation.as_path(),
        "deployment stack launch transport locator reconciliation",
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
        "reconciliation_path": args.reconciliation.display().to_string(),
        "handoff_path": read_string(&reconciliation, "handoff_path", args.reconciliation.as_path())?,
        "locator_path": read_string(&reconciliation, "locator_path", args.reconciliation.as_path())?,
        "reconciliation_status": read_string(&reconciliation, "reconciliation_status", args.reconciliation.as_path())?,
        "desired": read_object(&reconciliation, "desired", args.reconciliation.as_path())?,
        "current": read_object(&reconciliation, "current", args.reconciliation.as_path())?,
        "recorded_at_unix_ms": args.recorded_at_unix_ms,
    });
    history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("history entries must be an array")
        .push(entry);

    write_json_value(&args.out_path, &history)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_transport_locator_reconciliation_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    );
    Ok(())
}

fn parse_args<I>(
    args: I,
) -> Result<Args, DeploymentStackLaunchTransportLocatorReconciliationHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut reconciliation = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--reconciliation" => {
                reconciliation = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--reconciliation",
                )?))
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--reconciliation=") => {
                reconciliation = Some(PathBuf::from(
                    arg.trim_start_matches("--reconciliation=").to_string(),
                ))
            }
            _ if arg.starts_with("--event=") => {
                event = Some(arg.trim_start_matches("--event=").to_string())
            }
            _ if arg.starts_with("--recorded-at-unix-ms=") => recorded_at_unix_ms = Some(
                arg.trim_start_matches("--recorded-at-unix-ms=")
                    .parse::<u64>()
                    .map_err(|_| {
                        DeploymentStackLaunchTransportLocatorReconciliationHistoryError::InvalidArg(
                            format!("invalid value for --recorded-at-unix-ms\n{}", usage()),
                        )
                    })?,
            ),
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DeploymentStackLaunchTransportLocatorReconciliationHistoryError::InvalidArg(
                        format!(
                            "unknown argument for deploy record-launch-transport-locator-reconciliation-history: {arg}\n{}",
                            usage()
                        ),
                    ),
                );
            }
        }
    }

    Ok(Args {
        reconciliation: reconciliation.ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorReconciliationHistoryError::InvalidArg(format!(
                "missing value for --reconciliation\n{}",
                usage()
            ))
        })?,
        event: event.ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorReconciliationHistoryError::InvalidArg(format!(
                "missing value for --event\n{}",
                usage()
            ))
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorReconciliationHistoryError::InvalidArg(format!(
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
) -> Result<T, DeploymentStackLaunchTransportLocatorReconciliationHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchTransportLocatorReconciliationHistoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchTransportLocatorReconciliationHistoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<
    serde_json::Map<String, Value>,
    DeploymentStackLaunchTransportLocatorReconciliationHistoryError,
> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentStackLaunchTransportLocatorReconciliationHistoryError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentStackLaunchTransportLocatorReconciliationHistoryError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn load_history(
    path: &Path,
) -> Result<Value, DeploymentStackLaunchTransportLocatorReconciliationHistoryError> {
    let text = fs::read_to_string(path)?;
    let history: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentStackLaunchTransportLocatorReconciliationHistoryError::Parse(format!(
            "parse deployment stack launch transport locator reconciliation history `{}`: {err}",
            path.display()
        ))
    })?;
    history
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorReconciliationHistoryError::Parse(format!(
                "deployment stack launch transport locator reconciliation history `{}` must contain an `entries` array",
                path.display()
            ))
        })?;
    Ok(history)
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentStackLaunchTransportLocatorReconciliationHistoryError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorReconciliationHistoryError::Parse(format!(
                "deployment stack launch transport locator reconciliation `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_object(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<Value, DeploymentStackLaunchTransportLocatorReconciliationHistoryError> {
    object.get(key).cloned().ok_or_else(|| {
        DeploymentStackLaunchTransportLocatorReconciliationHistoryError::Parse(format!(
            "deployment stack launch transport locator reconciliation `{}` missing field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy record-launch-transport-locator-reconciliation-history --reconciliation PATH --event NAME --recorded-at-unix-ms MS [--out PATH]"
}
