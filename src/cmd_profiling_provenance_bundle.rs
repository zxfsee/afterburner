use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const PROFILING_PROVENANCE_EVIDENCE_BUNDLE_PATH: &str =
    "artifacts/profiling/profiling_provenance_evidence_bundle.json";

#[derive(Debug)]
enum ProfilingProvenanceBundleError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for ProfilingProvenanceBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProfilingProvenanceBundleError {}

impl From<std::io::Error> for ProfilingProvenanceBundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for ProfilingProvenanceBundleError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize profiling provenance bundle json: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    snapshot: PathBuf,
    summary: PathBuf,
    captured_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), ProfilingProvenanceBundleError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let snapshot = load_json_object(args.snapshot.as_path(), "profiling environment snapshot")?;
    let summary = load_json_object(args.summary.as_path(), "profiling hotspot summary")?;

    let bundle = json!({
        "schema_version": "1",
        "snapshot_path": args.snapshot.display().to_string(),
        "summary_path": args.summary.display().to_string(),
        "captured_at_unix_ms": args.captured_at_unix_ms,
        "artifact_version": read_string(&summary, "artifact_version", args.summary.as_path())?,
        "profile_kind": read_string(&summary, "profile_kind", args.summary.as_path())?,
        "profiler": read_string(&snapshot, "profiler", args.snapshot.as_path())?,
        "backend": read_string(&summary, "backend", args.summary.as_path())?,
        "evidence_count": 2,
        "evidence_sources": [
            {
                "artifact_path": args.snapshot.display().to_string(),
                "artifact_role": "profiling_environment_snapshot",
                "observed_at_unix_ms": args.captured_at_unix_ms
            },
            {
                "artifact_path": args.summary.display().to_string(),
                "artifact_role": "profiling_hotspot_summary",
                "observed_at_unix_ms": args.captured_at_unix_ms
            }
        ]
    });

    write_json_value(&args.out_path, &bundle)?;
    emit_json_artifact_written(
        "profiling_cli",
        "profiling_provenance_evidence_bundle_written",
        "bundle_path",
        &args.out_path,
        "bundle",
        bundle,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, ProfilingProvenanceBundleError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut snapshot = None::<PathBuf>;
    let mut summary = None::<PathBuf>;
    let mut captured_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(PROFILING_PROVENANCE_EVIDENCE_BUNDLE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--snapshot" => {
                let value: String = parse_value(&mut args, "--snapshot")?;
                snapshot = Some(PathBuf::from(value));
            }
            "--summary" => {
                let value: String = parse_value(&mut args, "--summary")?;
                summary = Some(PathBuf::from(value));
            }
            "--captured-at-unix-ms" => {
                captured_at_unix_ms = Some(parse_value(&mut args, "--captured-at-unix-ms")?)
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--snapshot=") => {
                snapshot = Some(PathBuf::from(
                    arg.trim_start_matches("--snapshot=").to_string(),
                ))
            }
            _ if arg.starts_with("--summary=") => {
                summary = Some(PathBuf::from(
                    arg.trim_start_matches("--summary=").to_string(),
                ))
            }
            _ if arg.starts_with("--captured-at-unix-ms=") => {
                captured_at_unix_ms = Some(
                    arg.trim_start_matches("--captured-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            ProfilingProvenanceBundleError::InvalidArg(format!(
                                "invalid value for --captured-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(ProfilingProvenanceBundleError::InvalidArg(format!(
                    "unknown argument for profile provenance-bundle: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        snapshot: snapshot.ok_or_else(|| {
            ProfilingProvenanceBundleError::InvalidArg(format!(
                "missing value for --snapshot\n{}",
                usage()
            ))
        })?,
        summary: summary.ok_or_else(|| {
            ProfilingProvenanceBundleError::InvalidArg(format!(
                "missing value for --summary\n{}",
                usage()
            ))
        })?,
        captured_at_unix_ms: captured_at_unix_ms.ok_or_else(|| {
            ProfilingProvenanceBundleError::InvalidArg(format!(
                "missing value for --captured-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, ProfilingProvenanceBundleError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        ProfilingProvenanceBundleError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        ProfilingProvenanceBundleError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, ProfilingProvenanceBundleError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        ProfilingProvenanceBundleError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        ProfilingProvenanceBundleError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, ProfilingProvenanceBundleError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            ProfilingProvenanceBundleError::Parse(format!(
                "profiling artifact `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner profile provenance-bundle --snapshot PATH --summary PATH --captured-at-unix-ms N [--out PATH]"
}
