use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const PROFILING_PROVENANCE_RECEIPT_PATH: &str =
    "artifacts/profiling/profiling_provenance_receipt.json";

#[derive(Debug)]
enum ProfilingProvenanceReceiptError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for ProfilingProvenanceReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProfilingProvenanceReceiptError {}

impl From<std::io::Error> for ProfilingProvenanceReceiptError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for ProfilingProvenanceReceiptError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize profiling provenance receipt json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    snapshot: PathBuf,
    refresh_receipt: Option<PathBuf>,
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

fn run_inner<I>(args: I) -> Result<(), ProfilingProvenanceReceiptError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let snapshot = load_json_object(args.snapshot.as_path(), "profiling environment snapshot")?;
    let profiler_version = read_string(&snapshot, "profiler_version", args.snapshot.as_path())?;
    let profile_kind = read_string(&snapshot, "profile_kind", args.snapshot.as_path())?;

    let mut receipt = json!({
        "schema_version": "1",
        "snapshot_path": args.snapshot.display().to_string(),
        "profile_kind": profile_kind,
        "profiler_version": profiler_version,
        "captured_at_unix_ms": args.captured_at_unix_ms
    });

    if let Some(refresh_receipt_path) = &args.refresh_receipt {
        receipt
            .as_object_mut()
            .expect("receipt must be object")
            .insert(
                "refresh_receipt_path".to_string(),
                Value::from(refresh_receipt_path.display().to_string()),
            );
    }

    write_json_value(&args.out_path, &receipt)?;
    emit_json_artifact_written(
        "profiling_cli",
        "profiling_provenance_receipt_written",
        "receipt_path",
        &args.out_path,
        "receipt",
        receipt,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, ProfilingProvenanceReceiptError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut snapshot = None::<PathBuf>;
    let mut refresh_receipt = None::<PathBuf>;
    let mut captured_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(PROFILING_PROVENANCE_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--snapshot" => {
                let value: String = parse_value(&mut args, "--snapshot")?;
                snapshot = Some(PathBuf::from(value));
            }
            "--refresh-receipt" => {
                let value: String = parse_value(&mut args, "--refresh-receipt")?;
                refresh_receipt = Some(PathBuf::from(value));
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
            _ if arg.starts_with("--refresh-receipt=") => {
                refresh_receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--refresh-receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--captured-at-unix-ms=") => {
                captured_at_unix_ms = Some(
                    arg.trim_start_matches("--captured-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            ProfilingProvenanceReceiptError::InvalidArg(format!(
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
                return Err(ProfilingProvenanceReceiptError::InvalidArg(format!(
                    "unknown argument for profile provenance-receipt: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        snapshot: snapshot.ok_or_else(|| {
            ProfilingProvenanceReceiptError::InvalidArg(format!(
                "missing value for --snapshot\n{}",
                usage()
            ))
        })?,
        refresh_receipt,
        captured_at_unix_ms: captured_at_unix_ms.ok_or_else(|| {
            ProfilingProvenanceReceiptError::InvalidArg(format!(
                "missing value for --captured-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, ProfilingProvenanceReceiptError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        ProfilingProvenanceReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        ProfilingProvenanceReceiptError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, ProfilingProvenanceReceiptError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        ProfilingProvenanceReceiptError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        ProfilingProvenanceReceiptError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, ProfilingProvenanceReceiptError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            ProfilingProvenanceReceiptError::Parse(format!(
                "profiling environment snapshot `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner profile provenance-receipt --snapshot PATH [--refresh-receipt PATH] --captured-at-unix-ms N [--out PATH]"
}
