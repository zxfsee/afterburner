use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/deploy/kube_rs_gpu_lease_pointer.json";

#[derive(Debug)]
enum KubeRsLeasePointerError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for KubeRsLeasePointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for KubeRsLeasePointerError {}

impl From<std::io::Error> for KubeRsLeasePointerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for KubeRsLeasePointerError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize kube-rs gpu lease pointer json: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    reconciliation: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), KubeRsLeasePointerError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let reconciliation = load_json_object(
        args.reconciliation.as_path(),
        "kube-rs gpu lease reconciliation",
    )?;

    let pointer = json!({
        "schema_version": "1",
        "reconciliation_path": args.reconciliation.display().to_string(),
        "lease_id": read_string(&reconciliation, "lease_id", args.reconciliation.as_path())?,
        "job_id": read_string(&reconciliation, "job_id", args.reconciliation.as_path())?,
        "node_id": read_string(&reconciliation, "node_id", args.reconciliation.as_path())?,
        "gpu_ids": read_string_array(&reconciliation, "gpu_ids", args.reconciliation.as_path())?,
        "checkpoint_mode": read_string(&reconciliation, "checkpoint_mode", args.reconciliation.as_path())?,
        "namespace": read_string(&reconciliation, "namespace", args.reconciliation.as_path())?,
        "resource_name": read_string(&reconciliation, "resource_name", args.reconciliation.as_path())?,
        "desired_state": read_string(&reconciliation, "desired_state", args.reconciliation.as_path())?,
    });

    write_json_value(&args.out_path, &pointer)?;
    emit_json_artifact_written(
        "deploy_cli",
        "kube_rs_gpu_lease_pointer_written",
        "pointer_path",
        &args.out_path,
        "pointer",
        pointer,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, KubeRsLeasePointerError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut reconciliation = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--reconciliation" => {
                reconciliation = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--reconciliation",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--reconciliation=") => {
                reconciliation = Some(PathBuf::from(
                    arg.trim_start_matches("--reconciliation=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(KubeRsLeasePointerError::InvalidArg(format!(
                    "unknown argument for deploy point-kube-rs-lease: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        reconciliation: reconciliation.ok_or_else(|| {
            KubeRsLeasePointerError::InvalidArg(format!(
                "missing value for --reconciliation\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, KubeRsLeasePointerError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        KubeRsLeasePointerError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        KubeRsLeasePointerError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, KubeRsLeasePointerError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        KubeRsLeasePointerError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        KubeRsLeasePointerError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, KubeRsLeasePointerError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            KubeRsLeasePointerError::Parse(format!(
                "kube-rs gpu lease reconciliation `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_string_array(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<Vec<String>, KubeRsLeasePointerError> {
    object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            KubeRsLeasePointerError::Parse(format!(
                "kube-rs gpu lease reconciliation `{}` missing array field `{key}`",
                path.display()
            ))
        })?
        .iter()
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                KubeRsLeasePointerError::Parse(format!(
                    "kube-rs gpu lease reconciliation `{}` field `{key}` must contain only strings",
                    path.display()
                ))
            })
        })
        .collect()
}

fn usage() -> &'static str {
    "usage: afterburner deploy point-kube-rs-lease --reconciliation PATH [--out PATH]"
}
