use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/train/distributed_shard_lineage_locator_history.json";

#[derive(Debug)]
enum DistributedShardLineageLocatorHistoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedShardLineageLocatorHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedShardLineageLocatorHistoryError {}

impl From<std::io::Error> for DistributedShardLineageLocatorHistoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DistributedShardLineageLocatorHistoryError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize distributed shard lineage locator history json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    pointer: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DistributedShardLineageLocatorHistoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let pointer = load_json_object(
        args.pointer.as_path(),
        "distributed shard lineage locator pointer",
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
        "pointer_path": args.pointer.display().to_string(),
        "locator_path": read_string(&pointer, "locator_path", args.pointer.as_path())?,
        "shard_id": read_string(&pointer, "shard_id", args.pointer.as_path())?,
        "source": read_string(&pointer, "source", args.pointer.as_path())?,
        "source_revision": read_string(&pointer, "source_revision", args.pointer.as_path())?,
        "checkpoint_group": read_string(&pointer, "checkpoint_group", args.pointer.as_path())?,
        "recorded_at_unix_ms": args.recorded_at_unix_ms,
    });
    history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("history entries must be an array")
        .push(entry);

    write_json_value(&args.out_path, &history)?;
    emit_json_artifact_written(
        "train_cli",
        "distributed_shard_lineage_locator_history_written",
        "history_path",
        &args.out_path,
        "history",
        history,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedShardLineageLocatorHistoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut pointer = None::<PathBuf>;
    let mut event = None::<String>;
    let mut recorded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pointer" => {
                let value: String = parse_value(&mut args, "--pointer")?;
                pointer = Some(PathBuf::from(value));
            }
            "--event" => event = Some(parse_value(&mut args, "--event")?),
            "--recorded-at-unix-ms" => {
                recorded_at_unix_ms = Some(parse_value(&mut args, "--recorded-at-unix-ms")?)
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--pointer=") => {
                pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--pointer=").to_string(),
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
                            DistributedShardLineageLocatorHistoryError::InvalidArg(format!(
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
                return Err(DistributedShardLineageLocatorHistoryError::InvalidArg(
                    format!(
                        "unknown argument for lineage record-locator-history: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        pointer: pointer.ok_or_else(|| {
            DistributedShardLineageLocatorHistoryError::InvalidArg(format!(
                "missing value for --pointer\n{}",
                usage()
            ))
        })?,
        event: event.ok_or_else(|| {
            DistributedShardLineageLocatorHistoryError::InvalidArg(format!(
                "missing value for --event\n{}",
                usage()
            ))
        })?,
        recorded_at_unix_ms: recorded_at_unix_ms.ok_or_else(|| {
            DistributedShardLineageLocatorHistoryError::InvalidArg(format!(
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
) -> Result<T, DistributedShardLineageLocatorHistoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedShardLineageLocatorHistoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedShardLineageLocatorHistoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_kind_history_helpers!(DistributedShardLineageLocatorHistoryError, "distributed shard lineage locator pointer", "distributed shard lineage locator history");

fn usage() -> &'static str {
    "usage: afterburner lineage record-locator-history --pointer PATH --event NAME --recorded-at-unix-ms MS [--out PATH]"
}
