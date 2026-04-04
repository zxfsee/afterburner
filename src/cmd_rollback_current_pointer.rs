use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use afterburner::rollout_current_pointer::{
    default_current_pointer_path, default_previous_current_version_path, rollback_current_pointer,
};
use serde_json::json;

const ROLLBACK_RECORD_PATH: &str = "artifacts/deploy/inference_current_pointer_rollback.json";

#[derive(Debug)]
enum RollbackCurrentPointerError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for RollbackCurrentPointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for RollbackCurrentPointerError {}

impl From<std::io::Error> for RollbackCurrentPointerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for RollbackCurrentPointerError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize current-pointer rollback json: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    current_pointer: PathBuf,
    previous_version_path: PathBuf,
    out_record: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), RollbackCurrentPointerError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let record = rollback_current_pointer(
        args.current_pointer.as_path(),
        args.previous_version_path.as_path(),
    )
    .map_err(RollbackCurrentPointerError::Parse)?;

    let payload = json!({
        "schema_version": "1",
        "current_pointer_path": record.current_pointer_path,
        "previous_version_path": record.previous_version_path,
        "previous_version": record.previous_version,
        "restored_version": record.restored_version,
    });
    write_json_value(&args.out_record, &payload)?;
    emit_json_artifact_written(
        "deploy_cli",
        "inference_current_pointer_rolled_back",
        "record_path",
        &args.out_record,
        "record",
        payload,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, RollbackCurrentPointerError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut current_pointer = default_current_pointer_path();
    let mut previous_version_path = default_previous_current_version_path();
    let mut out_record = PathBuf::from(ROLLBACK_RECORD_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--current-pointer" => {
                current_pointer =
                    PathBuf::from(parse_value::<String, _>(&mut args, "--current-pointer")?)
            }
            "--previous-version-file" => {
                previous_version_path = PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--previous-version-file",
                )?)
            }
            "--out-record" => {
                out_record = PathBuf::from(parse_value::<String, _>(&mut args, "--out-record")?)
            }
            _ if arg.starts_with("--current-pointer=") => {
                current_pointer =
                    PathBuf::from(arg.trim_start_matches("--current-pointer=").to_string())
            }
            _ if arg.starts_with("--previous-version-file=") => {
                previous_version_path = PathBuf::from(
                    arg.trim_start_matches("--previous-version-file=")
                        .to_string(),
                )
            }
            _ if arg.starts_with("--out-record=") => {
                out_record = PathBuf::from(arg.trim_start_matches("--out-record=").to_string())
            }
            _ => {
                return Err(RollbackCurrentPointerError::InvalidArg(format!(
                    "unknown argument for rollback current-pointer: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        current_pointer,
        previous_version_path,
        out_record,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, RollbackCurrentPointerError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        RollbackCurrentPointerError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        RollbackCurrentPointerError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner rollback current-pointer [--current-pointer PATH] [--previous-version-file PATH] [--out-record PATH]"
}
