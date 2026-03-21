use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

use crate::cmd_profiling_environment_snapshot::detect_profiler_version;

const PROFILING_ENVIRONMENT_SNAPSHOT_PATH: &str =
    "artifacts/profiling/profiling_environment_snapshot.json";
const PROFILING_ENVIRONMENT_SNAPSHOT_REFRESH_PATH: &str =
    "artifacts/profiling/profiling_environment_snapshot_refresh.json";

#[derive(Debug)]
enum ProfilingEnvironmentSnapshotRefreshError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for ProfilingEnvironmentSnapshotRefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProfilingEnvironmentSnapshotRefreshError {}

impl From<std::io::Error> for ProfilingEnvironmentSnapshotRefreshError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    current_snapshot: PathBuf,
    profiler_path: String,
    captured_at_unix_ms: i64,
    out_path: PathBuf,
    refresh_receipt_path: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), ProfilingEnvironmentSnapshotRefreshError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let current = load_json_object(
        args.current_snapshot.as_path(),
        "profiling environment snapshot",
    )?;

    let profile_kind = read_string(&current, "profile_kind", args.current_snapshot.as_path())?;
    let profiler = read_string(&current, "profiler", args.current_snapshot.as_path())?;
    let previous_profiler_path =
        read_string(&current, "profiler_path", args.current_snapshot.as_path())?;
    let previous_profiler_version = read_string(
        &current,
        "profiler_version",
        args.current_snapshot.as_path(),
    )?;
    let refreshed_profiler_version = detect_profiler_version(args.profiler_path.as_str())
        .map_err(|err| ProfilingEnvironmentSnapshotRefreshError::Parse(err.to_string()))?;

    let refreshed_snapshot = json!({
        "schema_version": "1",
        "profile_kind": profile_kind,
        "profiler": profiler,
        "profiler_path": args.profiler_path,
        "profiler_version": refreshed_profiler_version,
        "host_os": std::env::consts::OS,
        "host_arch": std::env::consts::ARCH,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let snapshot_text = serde_json::to_string_pretty(&refreshed_snapshot).map_err(|err| {
        ProfilingEnvironmentSnapshotRefreshError::Parse(format!(
            "serialize refreshed profiling environment snapshot json: {err}"
        ))
    })?;
    fs::write(&args.out_path, snapshot_text)?;

    let refresh_receipt = json!({
        "schema_version": "1",
        "previous_snapshot_path": args.current_snapshot.display().to_string(),
        "refreshed_snapshot_path": args.out_path.display().to_string(),
        "profile_kind": read_string(&current, "profile_kind", args.current_snapshot.as_path())?,
        "profiler": read_string(&current, "profiler", args.current_snapshot.as_path())?,
        "previous_profiler_path": previous_profiler_path,
        "refreshed_profiler_path": args.profiler_path,
        "previous_profiler_version": previous_profiler_version,
        "refreshed_profiler_version": refreshed_snapshot
            .get("profiler_version")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        "captured_at_unix_ms": args.captured_at_unix_ms,
    });

    if let Some(parent) = args.refresh_receipt_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let receipt_text = serde_json::to_string_pretty(&refresh_receipt).map_err(|err| {
        ProfilingEnvironmentSnapshotRefreshError::Parse(format!(
            "serialize profiling environment snapshot refresh json: {err}"
        ))
    })?;
    fs::write(&args.refresh_receipt_path, receipt_text)?;

    emit_event(
        "info",
        "profiling_cli",
        "profiling_environment_snapshot_refreshed",
        json!({
            "refresh_receipt_path": args.refresh_receipt_path.display().to_string(),
            "refresh_receipt": refresh_receipt,
        }),
    );
    Ok(())
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, ProfilingEnvironmentSnapshotRefreshError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        ProfilingEnvironmentSnapshotRefreshError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        ProfilingEnvironmentSnapshotRefreshError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, ProfilingEnvironmentSnapshotRefreshError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            ProfilingEnvironmentSnapshotRefreshError::Parse(format!(
                "profiling environment snapshot `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn parse_args<I>(args: I) -> Result<Args, ProfilingEnvironmentSnapshotRefreshError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut current_snapshot = None::<PathBuf>;
    let mut profiler_path = None::<String>;
    let mut captured_at_unix_ms = None::<i64>;
    let mut out_path = PathBuf::from(PROFILING_ENVIRONMENT_SNAPSHOT_PATH);
    let mut refresh_receipt_path = PathBuf::from(PROFILING_ENVIRONMENT_SNAPSHOT_REFRESH_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--current-snapshot" => {
                let value: String = parse_value(&mut args, "--current-snapshot")?;
                current_snapshot = Some(PathBuf::from(value));
            }
            "--profiler-path" => profiler_path = Some(parse_value(&mut args, "--profiler-path")?),
            "--captured-at-unix-ms" => {
                captured_at_unix_ms = Some(parse_value(&mut args, "--captured-at-unix-ms")?)
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            "--refresh-receipt" => {
                let value: String = parse_value(&mut args, "--refresh-receipt")?;
                refresh_receipt_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--current-snapshot=") => {
                current_snapshot = Some(PathBuf::from(
                    arg.trim_start_matches("--current-snapshot=").to_string(),
                ))
            }
            _ if arg.starts_with("--profiler-path=") => {
                profiler_path = Some(arg.trim_start_matches("--profiler-path=").to_string())
            }
            _ if arg.starts_with("--captured-at-unix-ms=") => {
                captured_at_unix_ms = Some(
                    arg.trim_start_matches("--captured-at-unix-ms=")
                        .parse()
                        .map_err(|_| {
                            ProfilingEnvironmentSnapshotRefreshError::InvalidArg(format!(
                                "invalid value for --captured-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ if arg.starts_with("--refresh-receipt=") => {
                refresh_receipt_path =
                    PathBuf::from(arg.trim_start_matches("--refresh-receipt=").to_string())
            }
            _ => {
                return Err(ProfilingEnvironmentSnapshotRefreshError::InvalidArg(
                    format!(
                        "unknown argument for profile refresh-environment-snapshot: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        current_snapshot: current_snapshot.ok_or_else(|| {
            ProfilingEnvironmentSnapshotRefreshError::InvalidArg(format!(
                "missing value for --current-snapshot\n{}",
                usage()
            ))
        })?,
        profiler_path: require_non_empty(profiler_path, "--profiler-path")?,
        captured_at_unix_ms: captured_at_unix_ms.ok_or_else(|| {
            ProfilingEnvironmentSnapshotRefreshError::InvalidArg(format!(
                "missing value for --captured-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
        refresh_receipt_path,
    })
}

fn require_non_empty(
    value: Option<String>,
    flag: &str,
) -> Result<String, ProfilingEnvironmentSnapshotRefreshError> {
    let value = value.ok_or_else(|| {
        ProfilingEnvironmentSnapshotRefreshError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    if value.trim().is_empty() {
        return Err(ProfilingEnvironmentSnapshotRefreshError::InvalidArg(
            format!("{flag} must be non-empty\n{}", usage()),
        ));
    }
    Ok(value)
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, ProfilingEnvironmentSnapshotRefreshError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        ProfilingEnvironmentSnapshotRefreshError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        ProfilingEnvironmentSnapshotRefreshError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner profile refresh-environment-snapshot --current-snapshot PATH --profiler-path PATH --captured-at-unix-ms N [--out PATH] [--refresh-receipt PATH]"
}
