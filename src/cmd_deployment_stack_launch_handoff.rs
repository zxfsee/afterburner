use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEPLOYMENT_STACK_LAUNCH_EVIDENCE_HANDOFF_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_evidence_handoff.json";

#[derive(Debug)]
enum DeploymentStackLaunchHandoffError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchHandoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchHandoffError {}

impl From<std::io::Error> for DeploymentStackLaunchHandoffError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchHandoffError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch evidence handoff json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    bundle: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchHandoffError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "deployment stack launch evidence bundle",
    )?;

    let handoff = json!({
        "schema_version": "1",
        "bundle_path": args.bundle.display().to_string(),
        "profile_name": read_string(&bundle, "profile_name", args.bundle.as_path())?,
        "deploy_hostname": read_string(&bundle, "deploy_hostname", args.bundle.as_path())?,
        "deployable_unit": read_string(&bundle, "deployable_unit", args.bundle.as_path())?,
        "rollout_entrypoint": read_string(&bundle, "rollout_entrypoint", args.bundle.as_path())?,
        "launched_at_unix_ms": read_u64(&bundle, "launched_at_unix_ms", args.bundle.as_path())?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path())?,
    });

    write_json_value(&args.out_path, &handoff)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_evidence_handoff_written",
        "handoff_path",
        &args.out_path,
        "handoff",
        handoff,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchHandoffError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut bundle = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_STACK_LAUNCH_EVIDENCE_HANDOFF_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bundle" => {
                let value: String = parse_value(&mut args, "--bundle")?;
                bundle = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--bundle=") => {
                bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentStackLaunchHandoffError::InvalidArg(format!(
                    "unknown argument for deploy stack-launch-handoff: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        bundle: bundle.ok_or_else(|| {
            DeploymentStackLaunchHandoffError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DeploymentStackLaunchHandoffError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchHandoffError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchHandoffError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentStackLaunchHandoffError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentStackLaunchHandoffError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentStackLaunchHandoffError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentStackLaunchHandoffError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentStackLaunchHandoffError::Parse(format!(
                "deployment stack launch evidence bundle `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, DeploymentStackLaunchHandoffError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        DeploymentStackLaunchHandoffError::Parse(format!(
            "deployment stack launch evidence bundle `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy stack-launch-handoff --bundle PATH [--out PATH]"
}
