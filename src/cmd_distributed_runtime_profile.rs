use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::{Map, Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/train/distributed_runtime_profile.json";

#[derive(Debug)]
enum DistributedRuntimeProfileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedRuntimeProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedRuntimeProfileError {}

impl From<std::io::Error> for DistributedRuntimeProfileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Args {
    benchmark_run_path: PathBuf,
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
        Ok(path) => {
            println!("{}", path.display());
            0
        }
        Err(err) => {
            eprintln!("{err}");
            2
        }
    }
}

fn run_inner<I>(args: I) -> Result<PathBuf, DistributedRuntimeProfileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let benchmark_run = read_json_object(&args.benchmark_run_path)?;
    let benchmark_status = required_string(&benchmark_run, "benchmark_status")?;
    if benchmark_status != "passed" {
        return Err(DistributedRuntimeProfileError::Parse(format!(
            "benchmark run must have benchmark_status=`passed`, got `{benchmark_status}`"
        )));
    }

    let profile = json!({
        "schema_version": required_string(&benchmark_run, "schema_version")?,
        "artifact_version": required_string(&benchmark_run, "artifact_version")?,
        "model_size": required_string(&benchmark_run, "model_size")?,
        "node_count": required_u64(&benchmark_run, "node_count")?,
        "gpus_per_node": required_u64(&benchmark_run, "gpus_per_node")?,
        "world_size": required_u64(&benchmark_run, "world_size")?,
        "parallelism": required_object_value(&benchmark_run, "parallelism")?,
        "metrics": required_object_value(&benchmark_run, "metrics")?,
        "runtime_settings": required_object_value(&benchmark_run, "runtime_settings")?,
        "environment_fingerprint": required_object_value(&benchmark_run, "environment_fingerprint")?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&profile).map_err(|err| {
        DistributedRuntimeProfileError::Parse(format!(
            "serialize distributed runtime profile: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "profile_cli",
        "distributed_runtime_profile_written",
        json!({
            "benchmark_run_path": args.benchmark_run_path.display().to_string(),
            "profile_path": args.out_path.display().to_string(),
            "profile": profile,
        }),
    );

    Ok(args.out_path)
}

fn read_json_object(path: &PathBuf) -> Result<Map<String, Value>, DistributedRuntimeProfileError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DistributedRuntimeProfileError::Parse(format!(
            "parse benchmark run json from {}: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DistributedRuntimeProfileError::Parse(format!(
            "benchmark run at {} must be a JSON object",
            path.display()
        ))
    })
}

fn required_value<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a Value, DistributedRuntimeProfileError> {
    object.get(key).ok_or_else(|| {
        DistributedRuntimeProfileError::Parse(format!(
            "benchmark run is missing required field `{key}`"
        ))
    })
}

fn required_string(
    object: &Map<String, Value>,
    key: &str,
) -> Result<String, DistributedRuntimeProfileError> {
    required_value(object, key)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            DistributedRuntimeProfileError::Parse(format!(
                "benchmark run field `{key}` must be a string"
            ))
        })
}

fn required_u64(
    object: &Map<String, Value>,
    key: &str,
) -> Result<u64, DistributedRuntimeProfileError> {
    required_value(object, key)?.as_u64().ok_or_else(|| {
        DistributedRuntimeProfileError::Parse(format!("benchmark run field `{key}` must be a u64"))
    })
}

fn required_object_value(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Value, DistributedRuntimeProfileError> {
    let value = required_value(object, key)?;
    if !value.is_object() {
        return Err(DistributedRuntimeProfileError::Parse(format!(
            "benchmark run field `{key}` must be an object"
        )));
    }
    Ok(value.clone())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedRuntimeProfileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut benchmark_run_path = None;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--benchmark-run" => {
                benchmark_run_path = Some(PathBuf::from(parse_string_value(
                    &mut args,
                    "--benchmark-run",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_string_value(&mut args, "--out")?),
            _ => {
                return Err(DistributedRuntimeProfileError::InvalidArg(format!(
                    "unknown argument for profile distributed-runtime-profile: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        benchmark_run_path: benchmark_run_path.ok_or_else(|| {
            DistributedRuntimeProfileError::InvalidArg(format!(
                "missing required argument --benchmark-run\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_string_value<I>(
    args: &mut std::iter::Peekable<I>,
    flag: &str,
) -> Result<String, DistributedRuntimeProfileError>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| {
        DistributedRuntimeProfileError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner profile distributed-runtime-profile --benchmark-run PATH [--out PATH]"
}
