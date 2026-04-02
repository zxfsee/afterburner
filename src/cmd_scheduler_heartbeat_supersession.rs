use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/deploy/gpu_scheduler_heartbeat_supersession.json";

#[derive(Debug)]
enum SchedulerHeartbeatSupersessionError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerHeartbeatSupersessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerHeartbeatSupersessionError {}

impl From<std::io::Error> for SchedulerHeartbeatSupersessionError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    previous_heartbeat: PathBuf,
    next_heartbeat: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SchedulerHeartbeatSupersessionError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let previous = load_json_object(args.previous_heartbeat.as_path(), "gpu scheduler heartbeat")?;
    let next = load_json_object(args.next_heartbeat.as_path(), "gpu scheduler heartbeat")?;

    let previous_job_id = read_string(&previous, "job_id", args.previous_heartbeat.as_path())?;
    let next_job_id = read_string(&next, "job_id", args.next_heartbeat.as_path())?;
    if previous_job_id != next_job_id {
        return Err(SchedulerHeartbeatSupersessionError::Parse(format!(
            "heartbeats must share one job_id, found `{previous_job_id}` and `{next_job_id}`"
        )));
    }
    let previous_lease_id = read_string(&previous, "lease_id", args.previous_heartbeat.as_path())?;
    let next_lease_id = read_string(&next, "lease_id", args.next_heartbeat.as_path())?;
    if previous_lease_id != next_lease_id {
        return Err(SchedulerHeartbeatSupersessionError::Parse(format!(
            "heartbeats must share one lease_id, found `{previous_lease_id}` and `{next_lease_id}`"
        )));
    }
    let previous_worker_id =
        read_string(&previous, "worker_id", args.previous_heartbeat.as_path())?;
    let next_worker_id = read_string(&next, "worker_id", args.next_heartbeat.as_path())?;
    if previous_worker_id != next_worker_id {
        return Err(SchedulerHeartbeatSupersessionError::Parse(format!(
            "heartbeats must share one worker_id, found `{previous_worker_id}` and `{next_worker_id}`"
        )));
    }

    let previous_state = read_string(&previous, "state", args.previous_heartbeat.as_path())?;
    let next_state = read_string(&next, "state", args.next_heartbeat.as_path())?;
    let previous_observed = read_u64(
        &previous,
        "observed_at_unix_ms",
        args.previous_heartbeat.as_path(),
    )?;
    let next_observed = read_u64(&next, "observed_at_unix_ms", args.next_heartbeat.as_path())?;
    let previous_progress = read_optional_string(&previous, "progress_marker");
    let next_progress = read_optional_string(&next, "progress_marker");
    if previous_state == next_state
        && previous_observed == next_observed
        && previous_progress == next_progress
    {
        return Err(SchedulerHeartbeatSupersessionError::Parse(format!(
            "next heartbeat `{}` must differ from previous heartbeat `{}` in state, observed_at_unix_ms, or progress_marker",
            args.next_heartbeat.display(),
            args.previous_heartbeat.display()
        )));
    }

    let mut supersession = json!({
        "schema_version": "1",
        "previous_heartbeat_path": args.previous_heartbeat.display().to_string(),
        "next_heartbeat_path": args.next_heartbeat.display().to_string(),
        "job_id": next_job_id,
        "lease_id": next_lease_id,
        "worker_id": next_worker_id,
        "previous_state": previous_state,
        "next_state": next_state,
        "previous_observed_at_unix_ms": previous_observed,
        "next_observed_at_unix_ms": next_observed,
        "superseded_at_unix_ms": args.superseded_at_unix_ms,
    });
    if let Some(previous_progress_marker) = previous_progress {
        supersession
            .as_object_mut()
            .expect("supersession must be object")
            .insert(
                "previous_progress_marker".to_string(),
                previous_progress_marker.into(),
            );
    }
    if let Some(next_progress_marker) = next_progress {
        supersession
            .as_object_mut()
            .expect("supersession must be object")
            .insert(
                "next_progress_marker".to_string(),
                next_progress_marker.into(),
            );
    }

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&supersession).map_err(|err| {
        SchedulerHeartbeatSupersessionError::Parse(format!(
            "serialize scheduler heartbeat supersession json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "gpu_scheduler_heartbeat_superseded",
        json!({
            "supersession_path": args.out_path.display().to_string(),
            "supersession": supersession,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerHeartbeatSupersessionError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut previous_heartbeat = None::<PathBuf>;
    let mut next_heartbeat = None::<PathBuf>;
    let mut superseded_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--previous-heartbeat" => {
                previous_heartbeat = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--previous-heartbeat",
                )?))
            }
            "--next-heartbeat" => {
                next_heartbeat = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--next-heartbeat",
                )?))
            }
            "--superseded-at-unix-ms" => {
                superseded_at_unix_ms = Some(parse_value(&mut args, "--superseded-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--previous-heartbeat=") => {
                previous_heartbeat = Some(PathBuf::from(
                    arg.trim_start_matches("--previous-heartbeat=").to_string(),
                ))
            }
            _ if arg.starts_with("--next-heartbeat=") => {
                next_heartbeat = Some(PathBuf::from(
                    arg.trim_start_matches("--next-heartbeat=").to_string(),
                ))
            }
            _ if arg.starts_with("--superseded-at-unix-ms=") => {
                superseded_at_unix_ms = Some(
                    arg.trim_start_matches("--superseded-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            SchedulerHeartbeatSupersessionError::InvalidArg(format!(
                                "invalid value for --superseded-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(SchedulerHeartbeatSupersessionError::InvalidArg(format!(
                    "unknown argument for deploy scheduler-heartbeat-supersede: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        previous_heartbeat: previous_heartbeat.ok_or_else(|| {
            SchedulerHeartbeatSupersessionError::InvalidArg(format!(
                "missing value for --previous-heartbeat\n{}",
                usage()
            ))
        })?,
        next_heartbeat: next_heartbeat.ok_or_else(|| {
            SchedulerHeartbeatSupersessionError::InvalidArg(format!(
                "missing value for --next-heartbeat\n{}",
                usage()
            ))
        })?,
        superseded_at_unix_ms: superseded_at_unix_ms.ok_or_else(|| {
            SchedulerHeartbeatSupersessionError::InvalidArg(format!(
                "missing value for --superseded-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, SchedulerHeartbeatSupersessionError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerHeartbeatSupersessionError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerHeartbeatSupersessionError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, SchedulerHeartbeatSupersessionError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        SchedulerHeartbeatSupersessionError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        SchedulerHeartbeatSupersessionError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<String, SchedulerHeartbeatSupersessionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            SchedulerHeartbeatSupersessionError::Parse(format!(
                "gpu scheduler heartbeat `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<u64, SchedulerHeartbeatSupersessionError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        SchedulerHeartbeatSupersessionError::Parse(format!(
            "gpu scheduler heartbeat `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn read_optional_string(object: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn usage() -> &'static str {
    "usage: afterburner debug deploy scheduler-heartbeat-supersede --previous-heartbeat PATH --next-heartbeat PATH --superseded-at-unix-ms MS [--out PATH]"
}
