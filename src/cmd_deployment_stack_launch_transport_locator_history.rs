use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_transport_locator_history.json";

#[derive(Debug)]
enum DeploymentStackLaunchTransportLocatorHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchTransportLocatorHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchTransportLocatorHistoryError {}

impl From<std::io::Error> for DeploymentStackLaunchTransportLocatorHistoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchTransportLocatorHistoryError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch transport locator history json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    locator: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchTransportLocatorHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let locator = load_json_object(
        args.locator.as_path(),
        "deployment stack launch transport locator",
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
        "locator_path": args.locator.display().to_string(),
        "handoff_path": read_string(&locator, "handoff_path", args.locator.as_path())?,
        "profile_name": read_string(&locator, "profile_name", args.locator.as_path())?,
        "deploy_hostname": read_string(&locator, "deploy_hostname", args.locator.as_path())?,
        "rollout_entrypoint": read_string(&locator, "rollout_entrypoint", args.locator.as_path())?,
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
        "deployment_stack_launch_transport_locator_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchTransportLocatorHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut locator = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--locator" => {
                locator = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--locator",
                )?))
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--locator=") => {
                locator = Some(PathBuf::from(
                    arg.trim_start_matches("--locator=").to_string(),
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
                            DeploymentStackLaunchTransportLocatorHistoryError::InvalidArg(format!(
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
                    DeploymentStackLaunchTransportLocatorHistoryError::InvalidArg(format!(
                        "unknown argument for deploy record-launch-transport-locator-history: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        locator: locator.ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorHistoryError::InvalidArg(format!(
                "missing value for --locator\n{}",
                usage()
            ))
        })?,
        event: event.ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorHistoryError::InvalidArg(format!(
                "missing value for --event\n{}",
                usage()
            ))
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorHistoryError::InvalidArg(format!(
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
) -> Result<T, DeploymentStackLaunchTransportLocatorHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchTransportLocatorHistoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchTransportLocatorHistoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentStackLaunchTransportLocatorHistoryError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentStackLaunchTransportLocatorHistoryError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentStackLaunchTransportLocatorHistoryError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn load_history(path: &Path) -> Result<Value, DeploymentStackLaunchTransportLocatorHistoryError> {
    let text = fs::read_to_string(path)?;
    let history: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentStackLaunchTransportLocatorHistoryError::Parse(format!(
            "parse deployment stack launch transport locator history `{}`: {err}",
            path.display()
        ))
    })?;
    history
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorHistoryError::Parse(format!(
                "deployment stack launch transport locator history `{}` must contain an `entries` array",
                path.display()
            ))
        })?;
    Ok(history)
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentStackLaunchTransportLocatorHistoryError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentStackLaunchTransportLocatorHistoryError::Parse(format!(
                "deployment stack launch transport locator `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner deploy record-launch-transport-locator-history --locator PATH --event NAME --recorded-at-unix-ms MS [--out PATH]"
}
