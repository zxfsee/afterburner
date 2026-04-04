use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use afterburner::infer::default_weights_path;

const DEFAULT_FLAMEGRAPH_PATH: &str = "artifacts/profiling/infer_flamegraph.svg";
const DEFAULT_SUMMARY_PATH: &str = "artifacts/profiling/infer_hotspot_summary.json";
const DEFAULT_SNAPSHOT_PATH: &str = "artifacts/profiling/profiling_environment_snapshot.json";
const DEFAULT_PROFILER_PATH: &str = "cargo-flamegraph";

#[derive(Debug)]
enum ProfilingInferError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
    Process(String),
}

impl std::fmt::Display for ProfilingInferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
            Self::Process(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProfilingInferError {}

impl From<std::io::Error> for ProfilingInferError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    artifact: PathBuf,
    backend: String,
    flamegraph_out: PathBuf,
    summary_out: PathBuf,
    snapshot_out: PathBuf,
    profiler_path: String,
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

fn run_inner<I>(args: I) -> Result<(), ProfilingInferError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    ensure_parent_dir(args.flamegraph_out.as_path())?;
    let profile_command =
        render_profile_command(args.flamegraph_out.as_path(), args.artifact.as_path());
    run_cargo_flamegraph(args.flamegraph_out.as_path(), args.artifact.as_path())?;
    afterburner::profiling_summary::write_infer_hotspot_summary(
        args.flamegraph_out.as_path(),
        args.summary_out.as_path(),
        args.artifact.as_path(),
        args.backend.as_str(),
        profile_command.as_str(),
    )
    .map_err(ProfilingInferError::Parse)?;
    crate::cmd_profiling_environment_snapshot::write_environment_snapshot(
        "infer",
        "cargo-flamegraph",
        args.profiler_path.as_str(),
        args.snapshot_out.as_path(),
    )
    .map_err(ProfilingInferError::Parse)?;
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, ProfilingInferError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut artifact = None::<PathBuf>;
    let mut backend = None::<String>;
    let mut flamegraph_out = PathBuf::from(DEFAULT_FLAMEGRAPH_PATH);
    let mut summary_out = PathBuf::from(DEFAULT_SUMMARY_PATH);
    let mut snapshot_out = PathBuf::from(DEFAULT_SNAPSHOT_PATH);
    let mut profiler_path = DEFAULT_PROFILER_PATH.to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => {
                artifact = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--artifact",
                )?))
            }
            "--backend" => backend = Some(parse_value(&mut args, "--backend")?),
            "--flamegraph-out" => {
                flamegraph_out =
                    PathBuf::from(parse_value::<String, _>(&mut args, "--flamegraph-out")?)
            }
            "--summary-out" => {
                summary_out = PathBuf::from(parse_value::<String, _>(&mut args, "--summary-out")?)
            }
            "--snapshot-out" => {
                snapshot_out = PathBuf::from(parse_value::<String, _>(&mut args, "--snapshot-out")?)
            }
            "--profiler-path" => profiler_path = parse_value(&mut args, "--profiler-path")?,
            _ if arg.starts_with("--artifact=") => {
                artifact = Some(PathBuf::from(
                    arg.trim_start_matches("--artifact=").to_string(),
                ))
            }
            _ if arg.starts_with("--backend=") => {
                backend = Some(arg.trim_start_matches("--backend=").to_string())
            }
            _ if arg.starts_with("--flamegraph-out=") => {
                flamegraph_out =
                    PathBuf::from(arg.trim_start_matches("--flamegraph-out=").to_string())
            }
            _ if arg.starts_with("--summary-out=") => {
                summary_out = PathBuf::from(arg.trim_start_matches("--summary-out=").to_string())
            }
            _ if arg.starts_with("--snapshot-out=") => {
                snapshot_out = PathBuf::from(arg.trim_start_matches("--snapshot-out=").to_string())
            }
            _ if arg.starts_with("--profiler-path=") => {
                profiler_path = arg.trim_start_matches("--profiler-path=").to_string()
            }
            _ => {
                return Err(ProfilingInferError::InvalidArg(format!(
                    "unknown argument for profile infer: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    let artifact = artifact.unwrap_or_else(default_weights_path);
    let backend = backend
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(crate::cmd_infer::resolve_backend);
    let profiler_path = profiler_path.trim().to_string();
    if profiler_path.is_empty() {
        return Err(ProfilingInferError::InvalidArg(format!(
            "--profiler-path must be non-empty\n{}",
            usage()
        )));
    }

    Ok(Args {
        artifact,
        backend,
        flamegraph_out,
        summary_out,
        snapshot_out,
        profiler_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, ProfilingInferError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        ProfilingInferError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        ProfilingInferError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn run_cargo_flamegraph(flamegraph_out: &Path, artifact: &Path) -> Result<(), ProfilingInferError> {
    let mut command = Command::new("cargo");
    command
        .arg("flamegraph")
        .arg("--dev")
        .arg("--deterministic")
        .arg("--bin")
        .arg("afterburner")
        .arg("-o")
        .arg(flamegraph_out)
        .arg("--")
        .arg("infer")
        .arg(artifact);

    if cfg!(target_os = "macos") {
        command.env_remove("DEVELOPER_DIR");
        command.env_remove("SDKROOT");
        command.env("XCTRACE", "/usr/bin/xctrace");
    }

    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        "no output".to_string()
    };
    Err(ProfilingInferError::Process(format!(
        "cargo flamegraph infer failed: {detail}"
    )))
}

fn ensure_parent_dir(path: &Path) -> Result<(), ProfilingInferError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn render_profile_command(flamegraph_out: &Path, artifact: &Path) -> String {
    let command = format!(
        "cargo flamegraph --dev --deterministic --bin afterburner -o {} -- infer {}",
        flamegraph_out.display(),
        artifact.display()
    );
    if cfg!(target_os = "macos") {
        format!("env -u DEVELOPER_DIR -u SDKROOT XCTRACE=/usr/bin/xctrace {command}")
    } else {
        command
    }
}

fn usage() -> &'static str {
    "usage: afterburner profile infer [--artifact PATH] [--backend NAME] [--flamegraph-out PATH] [--summary-out PATH] [--snapshot-out PATH] [--profiler-path PATH]"
}
