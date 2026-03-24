use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEPLOYMENT_VERIFICATION_EVIDENCE_HANDOFF_PATH: &str =
    "artifacts/deploy/deployment_verification_evidence_handoff.json";

#[derive(Debug)]
enum DeploymentVerificationHandoffError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentVerificationHandoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentVerificationHandoffError {}

impl From<std::io::Error> for DeploymentVerificationHandoffError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentVerificationHandoffError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment verification evidence handoff json: {err}"
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

fn run_inner<I>(args: I) -> Result<(), DeploymentVerificationHandoffError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "deployment verification evidence bundle",
    )?;

    let handoff = json!({
        "schema_version": "1",
        "bundle_path": args.bundle.display().to_string(),
        "artifact_version": read_string(&bundle, "artifact_version", args.bundle.as_path())?,
        "profile_name": read_string(&bundle, "profile_name", args.bundle.as_path())?,
        "verification_status": read_string(&bundle, "verification_status", args.bundle.as_path())?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path())?,
    });

    write_json_value(&args.out_path, &handoff)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_verification_evidence_handoff_written",
        "handoff_path",
        &args.out_path,
        "handoff",
        handoff,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentVerificationHandoffError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut bundle = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_VERIFICATION_EVIDENCE_HANDOFF_PATH);

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
                return Err(DeploymentVerificationHandoffError::InvalidArg(format!(
                    "unknown argument for deploy verification-handoff: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        bundle: bundle.ok_or_else(|| {
            DeploymentVerificationHandoffError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DeploymentVerificationHandoffError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentVerificationHandoffError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentVerificationHandoffError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DeploymentVerificationHandoffError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentVerificationHandoffError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentVerificationHandoffError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentVerificationHandoffError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentVerificationHandoffError::Parse(format!(
                "deployment verification evidence bundle `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<u64, DeploymentVerificationHandoffError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        DeploymentVerificationHandoffError::Parse(format!(
            "deployment verification evidence bundle `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy verification-handoff --bundle PATH [--out PATH]"
}
