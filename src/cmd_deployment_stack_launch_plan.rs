use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEPLOYMENT_STACK_LAUNCH_PLAN_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_plan.json";
const STACK_SCHEMA_VERSION: &str = "1";

#[derive(Debug)]
enum DeploymentStackLaunchPlanError {
    InvalidArg(String),
    Io(std::io::Error),
    Json(String),
    Contract(String),
}

impl std::fmt::Display for DeploymentStackLaunchPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Contract(err) => write!(f, "deployment stack launch contract error: {err}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchPlanError {}

impl From<std::io::Error> for DeploymentStackLaunchPlanError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchPlanError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Json(err.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    target_profile: PathBuf,
    stack_profile: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchPlanError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let target = load_json_object(&args.target_profile)?;
    let stack = load_json_object(&args.stack_profile)?;

    let profile_name = read_string(&target, "profile_name", args.target_profile.as_path())?;
    let deploy_hostname = read_string(&target, "deploy_hostname", args.target_profile.as_path())?;
    let ssh_user = read_string(&target, "ssh_user", args.target_profile.as_path())?;
    let system = read_string(&target, "system", args.target_profile.as_path())?;
    let artifact_root = read_string(&target, "artifact_root", args.target_profile.as_path())?;
    let activation_strategy = read_string(
        &target,
        "activation_strategy",
        args.target_profile.as_path(),
    )?;
    let deployable_unit = read_string(&stack, "deployable_unit", args.stack_profile.as_path())?;
    let service_entrypoint =
        read_string(&stack, "service_entrypoint", args.stack_profile.as_path())?;
    let rollout_entrypoint =
        read_relative_string(&stack, "rollout_entrypoint", args.stack_profile.as_path())?;
    let observability = read_object(&stack, "observability", args.stack_profile.as_path())?;
    let events_env = read_string(observability, "events_env", args.stack_profile.as_path())?;
    let default_events_path = read_relative_string(
        observability,
        "default_events_path",
        args.stack_profile.as_path(),
    )?;

    let target_schema_version =
        read_string(&target, "schema_version", args.target_profile.as_path())?;
    let stack_schema_version = read_string(&stack, "schema_version", args.stack_profile.as_path())?;
    if target_schema_version != STACK_SCHEMA_VERSION || stack_schema_version != STACK_SCHEMA_VERSION
    {
        return Err(DeploymentStackLaunchPlanError::Contract(format!(
            "target and stack profiles must both use schema_version `{STACK_SCHEMA_VERSION}`"
        )));
    }

    let plan = json!({
        "schema_version": STACK_SCHEMA_VERSION,
        "profile_name": profile_name,
        "deploy_hostname": deploy_hostname,
        "ssh_user": ssh_user,
        "system": system,
        "deployable_unit": deployable_unit,
        "activation_strategy": activation_strategy,
        "working_directory": artifact_root,
        "service_entrypoint": service_entrypoint,
        "rollout_entrypoint": PathBuf::from(&artifact_root).join(&rollout_entrypoint).display().to_string(),
        "required_env": ["PORT", events_env],
        "default_env": {
            "AFTERBURNER_OBS_JSONL_PATH": PathBuf::from(&artifact_root).join(&default_events_path).display().to_string()
        }
    });

    write_json_value(&args.out_path, &plan)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_plan_written",
        "plan_path",
        &args.out_path,
        "plan",
        plan,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchPlanError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut target_profile = None::<PathBuf>;
    let mut stack_profile = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_STACK_LAUNCH_PLAN_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target-profile" => {
                target_profile = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--target-profile",
                )?))
            }
            "--stack-profile" => {
                stack_profile = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--stack-profile",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--target-profile=") => {
                target_profile = Some(PathBuf::from(
                    arg.trim_start_matches("--target-profile=").to_string(),
                ))
            }
            _ if arg.starts_with("--stack-profile=") => {
                stack_profile = Some(PathBuf::from(
                    arg.trim_start_matches("--stack-profile=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentStackLaunchPlanError::InvalidArg(format!(
                    "unknown argument for deployment-stack-launch-plan: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        target_profile: target_profile.ok_or_else(|| {
            DeploymentStackLaunchPlanError::InvalidArg(format!(
                "missing value for --target-profile\n{}",
                usage()
            ))
        })?,
        stack_profile: stack_profile.ok_or_else(|| {
            DeploymentStackLaunchPlanError::InvalidArg(format!(
                "missing value for --stack-profile\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DeploymentStackLaunchPlanError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchPlanError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchPlanError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn load_json_object(
    path: &Path,
) -> Result<serde_json::Map<String, Value>, DeploymentStackLaunchPlanError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| DeploymentStackLaunchPlanError::Json(err.to_string()))?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentStackLaunchPlanError::Json(format!("{} must be an object", path.display()))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DeploymentStackLaunchPlanError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DeploymentStackLaunchPlanError::Contract(format!(
                "stack profile `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_object<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<&'a serde_json::Map<String, Value>, DeploymentStackLaunchPlanError> {
    object.get(key).and_then(Value::as_object).ok_or_else(|| {
        DeploymentStackLaunchPlanError::Contract(format!(
            "stack profile `{}` missing object field `{key}`",
            path.display()
        ))
    })
}

fn read_relative_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<PathBuf, DeploymentStackLaunchPlanError> {
    let value = read_string(object, key, path)?;
    let candidate = PathBuf::from(value.clone());
    if candidate.is_absolute() {
        return Err(DeploymentStackLaunchPlanError::Contract(format!(
            "stack profile `{}` field `{key}` must be relative, got `{value}`",
            path.display()
        )));
    }
    Ok(candidate)
}

fn usage() -> &'static str {
    "usage: afterburner deploy stack-launch-plan --target-profile PATH --stack-profile PATH [--out PATH]"
}
