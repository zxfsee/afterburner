use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/deploy/gpu_scheduler_heartbeat_pointer.json";

#[derive(Debug)]
enum SchedulerHeartbeatPointerError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatPointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatPointerError {}

impl From<std::io::Error> for SchedulerHeartbeatPointerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for SchedulerHeartbeatPointerError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize scheduler heartbeat pointer json: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    heartbeat: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatPointerError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let heartbeat = load_json_object(args.heartbeat.as_path(), "gpu scheduler heartbeat")?;

    let mut pointer = json!({
        "schema_version": "1",
        "heartbeat_path": args.heartbeat.display().to_string(),
        "job_id": read_string(&heartbeat, "job_id", args.heartbeat.as_path())?,
        "lease_id": read_string(&heartbeat, "lease_id", args.heartbeat.as_path())?,
        "worker_id": read_string(&heartbeat, "worker_id", args.heartbeat.as_path())?,
        "state": read_string(&heartbeat, "state", args.heartbeat.as_path())?,
        "observed_at_unix_ms": read_u64(&heartbeat, "observed_at_unix_ms", args.heartbeat.as_path())?,
    });

    if let Some(progress_marker) = heartbeat.get("progress_marker").and_then(Value::as_str) {
        pointer
            .as_object_mut()
            .expect("pointer must be object")
            .insert("progress_marker".to_string(), Value::from(progress_marker));
    }

    write_json_value(&args.out_path, &pointer)?;
    emit_json_artifact_written(
        "deploy_cli",
        "gpu_scheduler_heartbeat_pointer_written",
        "pointer_path",
        &args.out_path,
        "pointer",
        pointer,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatPointerError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut heartbeat = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--heartbeat" => {
                heartbeat = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--heartbeat",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--heartbeat=") => {
                heartbeat = Some(PathBuf::from(
                    arg.trim_start_matches("--heartbeat=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(SchedulerHeartbeatPointerError::InvalidArg(format!(
                    "unknown argument for deploy point-scheduler-heartbeat: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        heartbeat: heartbeat.ok_or_else(|| {
            SchedulerHeartbeatPointerError::InvalidArg(format!(
                "missing value for --heartbeat\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, SchedulerHeartbeatPointerError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatPointerError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatPointerError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

afterburner::define_command_input_json_non_empty_kind_helpers!(
    SchedulerHeartbeatPointerError,
    "gpu scheduler heartbeat"
);

fn usage() -> &'static str {
    "usage: afterburner debug deploy point-scheduler-heartbeat --heartbeat PATH [--out PATH]"
}
