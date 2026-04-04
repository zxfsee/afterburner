use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use afterburner::observability::emit_event;
use serde_json::json;

const PROFILING_ENVIRONMENT_SNAPSHOT_PATH: &str =
    "artifacts/profiling/profiling_environment_snapshot.json";

#[derive(Debug)]
enum ProfilingEnvironmentSnapshotError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for ProfilingEnvironmentSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProfilingEnvironmentSnapshotError {}

impl From<std::io::Error> for ProfilingEnvironmentSnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    profile_kind: String,
    profiler: String,
    profiler_path: String,
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

fn run_inner<I>(args: I) -> Result<(), ProfilingEnvironmentSnapshotError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    write_environment_snapshot(
        args.profile_kind.as_str(),
        args.profiler.as_str(),
        args.profiler_path.as_str(),
        args.out_path.as_path(),
    )
    .map_err(ProfilingEnvironmentSnapshotError::Parse)
}

pub(crate) fn write_environment_snapshot(
    profile_kind: &str,
    profiler: &str,
    profiler_path: &str,
    out_path: &Path,
) -> Result<(), String> {
    let profiler_version = detect_profiler_version(profiler_path)?;
    let snapshot = json!({
        "schema_version": "1",
        "profile_kind": profile_kind,
        "profiler": profiler,
        "profiler_path": profiler_path,
        "profiler_version": profiler_version,
        "host_os": std::env::consts::OS,
        "host_arch": std::env::consts::ARCH,
    });

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "create profiling environment snapshot directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    let text = serde_json::to_string_pretty(&snapshot)
        .map_err(|err| format!("serialize profiling environment snapshot json: {err}"))?;
    fs::write(out_path, text).map_err(|err| {
        format!(
            "write profiling environment snapshot `{}`: {err}",
            out_path.display()
        )
    })?;

    emit_event(
        "info",
        "profiling_cli",
        "profiling_environment_snapshot_written",
        json!({
            "snapshot_path": out_path.display().to_string(),
            "snapshot": snapshot,
        }),
    );
    Ok(())
}

pub(crate) fn detect_profiler_version(profiler_path: &str) -> Result<String, String> {
    let output = Command::new(profiler_path)
        .arg("--version")
        .output()
        .map_err(|err| format!("run profiler `{profiler_path}` --version: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "profiler `{profiler_path}` --version failed: {}",
            stderr.trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    let version = version.trim();
    if version.is_empty() {
        return Err(format!(
            "profiler `{profiler_path}` --version produced empty output"
        ));
    }
    Ok(version.to_string())
}

fn parse_args<I>(args: I) -> Result<Args, ProfilingEnvironmentSnapshotError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut profile_kind = None::<String>;
    let mut profiler = None::<String>;
    let mut profiler_path = None::<String>;
    let mut out_path = PathBuf::from(PROFILING_ENVIRONMENT_SNAPSHOT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--profile-kind" => profile_kind = Some(parse_value(&mut args, "--profile-kind")?),
            "--profiler" => profiler = Some(parse_value(&mut args, "--profiler")?),
            "--profiler-path" => profiler_path = Some(parse_value(&mut args, "--profiler-path")?),
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--profile-kind=") => {
                profile_kind = Some(arg.trim_start_matches("--profile-kind=").to_string())
            }
            _ if arg.starts_with("--profiler=") => {
                profiler = Some(arg.trim_start_matches("--profiler=").to_string())
            }
            _ if arg.starts_with("--profiler-path=") => {
                profiler_path = Some(arg.trim_start_matches("--profiler-path=").to_string())
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(ProfilingEnvironmentSnapshotError::InvalidArg(format!(
                    "unknown argument for profile environment-snapshot: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    let profile_kind = require_non_empty(profile_kind, "--profile-kind")?;
    let profiler = require_non_empty(profiler, "--profiler")?;
    let profiler_path = require_non_empty(profiler_path, "--profiler-path")?;

    Ok(Args {
        profile_kind,
        profiler,
        profiler_path,
        out_path,
    })
}

fn require_non_empty(
    value: Option<String>,
    flag: &str,
) -> Result<String, ProfilingEnvironmentSnapshotError> {
    let value = value.ok_or_else(|| {
        ProfilingEnvironmentSnapshotError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    if value.trim().is_empty() {
        return Err(ProfilingEnvironmentSnapshotError::InvalidArg(format!(
            "{flag} must be non-empty\n{}",
            usage()
        )));
    }
    Ok(value)
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, ProfilingEnvironmentSnapshotError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        ProfilingEnvironmentSnapshotError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        ProfilingEnvironmentSnapshotError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner profile environment-snapshot --profile-kind KIND --profiler NAME --profiler-path PATH [--out PATH]"
}
