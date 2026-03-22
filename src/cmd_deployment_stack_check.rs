use std::fs;
use std::path::{Component, Path, PathBuf};

use afterburner::observability::{
    OBS_JSONL_SINK_ENV, attach_trace_fields, current_trace_context, emit_event,
};
use serde_json::{Value, json};

const DEPLOYMENT_STACK_SCHEMA_VERSION: &str = "1";

#[derive(Debug)]
enum DeploymentStackCheckError {
    InvalidArg(String),
    Io(std::io::Error),
    Json(String),
    Contract(String),
}

impl std::fmt::Display for DeploymentStackCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Contract(err) => write!(f, "deployment stack contract error: {err}"),
        }
    }
}

impl std::error::Error for DeploymentStackCheckError {}

impl From<std::io::Error> for DeploymentStackCheckError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    target_profile: PathBuf,
    stack_profile: PathBuf,
    out_path: Option<PathBuf>,
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
        Ok(path) => {
            println!("{}", path.display());
            0
        }
        Err(err) => {
            eprintln!("{err}");
            2
        }
    }
}

fn run_inner<I>(args: I) -> Result<PathBuf, DeploymentStackCheckError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let trace = current_trace_context("deploy_cli");
    let target = load_json_object(&args.target_profile)?;
    let stack = load_json_object(&args.stack_profile)?;

    let profile_name = read_string(&target, "profile_name", args.target_profile.as_path())?;
    let deploy_hostname = read_string(&target, "deploy_hostname", args.target_profile.as_path())?;
    let artifact_root = read_string(&target, "artifact_root", args.target_profile.as_path())?;
    let activation_strategy = read_string(
        &target,
        "activation_strategy",
        args.target_profile.as_path(),
    )?;
    let target_schema_version =
        read_string(&target, "schema_version", args.target_profile.as_path())?;

    if target_schema_version != DEPLOYMENT_STACK_SCHEMA_VERSION {
        return Err(DeploymentStackCheckError::Contract(format!(
            "target profile schema_version must be `{DEPLOYMENT_STACK_SCHEMA_VERSION}`, got `{target_schema_version}`"
        )));
    }
    if activation_strategy != "current-pointer" {
        return Err(DeploymentStackCheckError::Contract(format!(
            "activation_strategy must be `current-pointer`, got `{activation_strategy}`"
        )));
    }

    let deployable_unit = read_string(&stack, "deployable_unit", args.stack_profile.as_path())?;
    let stack_schema_version = read_string(&stack, "schema_version", args.stack_profile.as_path())?;
    if stack_schema_version != DEPLOYMENT_STACK_SCHEMA_VERSION {
        return Err(DeploymentStackCheckError::Contract(format!(
            "stack profile schema_version must be `{DEPLOYMENT_STACK_SCHEMA_VERSION}`, got `{stack_schema_version}`"
        )));
    }

    let service_entrypoint =
        read_string(&stack, "service_entrypoint", args.stack_profile.as_path())?;
    let service_surface = read_object(&stack, "service_surface", args.stack_profile.as_path())?;
    let healthcheck_path = read_string(
        service_surface,
        "healthcheck_path",
        args.stack_profile.as_path(),
    )?;
    let infer_path = read_string(service_surface, "infer_path", args.stack_profile.as_path())?;
    let port_env = read_string(service_surface, "port_env", args.stack_profile.as_path())?;
    validate_service_path("service_surface.healthcheck_path", &healthcheck_path)?;
    validate_service_path("service_surface.infer_path", &infer_path)?;
    if port_env != "PORT" {
        return Err(DeploymentStackCheckError::Contract(format!(
            "service_surface.port_env must be `PORT`, got `{port_env}`"
        )));
    }

    let artifact_roots = read_object(&stack, "artifact_roots", args.stack_profile.as_path())?;
    let inference_root =
        read_relative_string(artifact_roots, "inference", args.stack_profile.as_path())?;
    let deploy_root = read_relative_string(artifact_roots, "deploy", args.stack_profile.as_path())?;
    let train_root = read_relative_string(artifact_roots, "train", args.stack_profile.as_path())?;
    let eval_root = read_relative_string(artifact_roots, "eval", args.stack_profile.as_path())?;

    let rollout_entrypoint =
        read_relative_string(&stack, "rollout_entrypoint", args.stack_profile.as_path())?;
    ensure_within_root(
        "rollout_entrypoint",
        rollout_entrypoint.as_path(),
        inference_root.as_path(),
    )?;

    let observability = read_object(&stack, "observability", args.stack_profile.as_path())?;
    let events_env = read_string(observability, "events_env", args.stack_profile.as_path())?;
    let default_events_path = read_relative_string(
        observability,
        "default_events_path",
        args.stack_profile.as_path(),
    )?;
    if events_env != OBS_JSONL_SINK_ENV {
        return Err(DeploymentStackCheckError::Contract(format!(
            "observability.events_env must be `{OBS_JSONL_SINK_ENV}`, got `{events_env}`"
        )));
    }
    ensure_within_root(
        "observability.default_events_path",
        default_events_path.as_path(),
        train_root.as_path(),
    )?;

    let artifact_root = PathBuf::from(artifact_root);
    let out_path = args.out_path.unwrap_or_else(|| {
        PathBuf::from("artifacts")
            .join("deploy")
            .join("deployment_stack_check.json")
    });

    let mut output = json!({
        "schema_version": DEPLOYMENT_STACK_SCHEMA_VERSION,
        "profile_name": profile_name,
        "deploy_hostname": deploy_hostname,
        "deployable_unit": deployable_unit,
        "activation_strategy": activation_strategy,
        "service_entrypoint": service_entrypoint,
        "artifact_root": artifact_root.display().to_string(),
        "artifact_roots": {
            "inference": artifact_root.join(&inference_root).display().to_string(),
            "deploy": artifact_root.join(&deploy_root).display().to_string(),
            "train": artifact_root.join(&train_root).display().to_string(),
            "eval": artifact_root.join(&eval_root).display().to_string(),
        },
        "rollout_entrypoint": artifact_root.join(&rollout_entrypoint).display().to_string(),
        "service_surface": {
            "healthcheck_path": healthcheck_path,
            "infer_path": infer_path,
            "port_env": port_env,
        },
        "observability": {
            "events_env": events_env,
            "default_events_path": artifact_root.join(&default_events_path).display().to_string(),
        }
    });
    attach_trace_fields(
        output
            .as_object_mut()
            .expect("deployment stack check output must be an object"),
        &trace,
    );

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &out_path,
        serde_json::to_string_pretty(&output)
            .map_err(|err| DeploymentStackCheckError::Json(err.to_string()))?,
    )?;
    emit_event(
        "info",
        "deploy_cli",
        "deployment_stack_check_written",
        json!({
            "check_path": out_path.display().to_string(),
            "profile_name": output.get("profile_name").and_then(Value::as_str).unwrap_or_default(),
            "traceparent": trace.traceparent,
            "trace_id": trace.trace_id,
        }),
    );
    Ok(out_path)
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackCheckError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut target_profile = None::<PathBuf>;
    let mut stack_profile = None::<PathBuf>;
    let mut out_path = None::<PathBuf>;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target-profile" => {
                target_profile = Some(PathBuf::from(parse_value(&mut args, "--target-profile")?))
            }
            "--stack-profile" => {
                stack_profile = Some(PathBuf::from(parse_value(&mut args, "--stack-profile")?))
            }
            "--out" => out_path = Some(PathBuf::from(parse_value(&mut args, "--out")?)),
            _ if arg.starts_with("--target-profile=") => {
                target_profile = Some(PathBuf::from(parse_inline_value(
                    arg.trim_start_matches("--target-profile="),
                    "--target-profile",
                )?))
            }
            _ if arg.starts_with("--stack-profile=") => {
                stack_profile = Some(PathBuf::from(parse_inline_value(
                    arg.trim_start_matches("--stack-profile="),
                    "--stack-profile",
                )?))
            }
            _ if arg.starts_with("--out=") => {
                out_path = Some(PathBuf::from(parse_inline_value(
                    arg.trim_start_matches("--out="),
                    "--out",
                )?))
            }
            _ => {
                return Err(DeploymentStackCheckError::InvalidArg(format!(
                    "unknown argument for deployment-stack-check: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        target_profile: target_profile.ok_or_else(|| {
            DeploymentStackCheckError::InvalidArg(format!(
                "missing value for --target-profile\n{}",
                usage()
            ))
        })?,
        stack_profile: stack_profile.ok_or_else(|| {
            DeploymentStackCheckError::InvalidArg(format!(
                "missing value for --stack-profile\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, DeploymentStackCheckError>
where
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackCheckError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    parse_inline_value(value.as_str(), flag)
}

fn parse_inline_value(value: &str, flag: &str) -> Result<String, DeploymentStackCheckError> {
    if value.trim().is_empty() {
        return Err(DeploymentStackCheckError::InvalidArg(format!(
            "{flag} must be non-empty\n{}",
            usage()
        )));
    }
    Ok(value.to_string())
}

fn load_json_object(
    path: &Path,
) -> Result<serde_json::Map<String, Value>, DeploymentStackCheckError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DeploymentStackCheckError::Json(format!("parse {}: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DeploymentStackCheckError::Json(format!("{} must contain a json object", path.display()))
    })
}

fn read_object<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<&'a serde_json::Map<String, Value>, DeploymentStackCheckError> {
    object.get(key).and_then(Value::as_object).ok_or_else(|| {
        DeploymentStackCheckError::Contract(format!(
            "{} must define object field `{key}`",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<String, DeploymentStackCheckError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| {
            DeploymentStackCheckError::Contract(format!(
                "{} must define non-empty string field `{key}`",
                path.display()
            ))
        })
}

fn read_relative_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<PathBuf, DeploymentStackCheckError> {
    let raw = read_string(object, key, path)?;
    validate_relative_path(key, &raw)
}

fn validate_relative_path(field: &str, raw: &str) -> Result<PathBuf, DeploymentStackCheckError> {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return Err(DeploymentStackCheckError::Contract(format!(
            "{field} must be a relative path, got `{raw}`"
        )));
    }
    if path.components().next().is_none() {
        return Err(DeploymentStackCheckError::Contract(format!(
            "{field} must not be empty"
        )));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(DeploymentStackCheckError::Contract(format!(
                    "{field} must not escape the artifact root, got `{raw}`"
                )));
            }
        }
    }
    Ok(path)
}

fn ensure_within_root(
    field: &str,
    path: &Path,
    root: &Path,
) -> Result<(), DeploymentStackCheckError> {
    if !path.starts_with(root) {
        return Err(DeploymentStackCheckError::Contract(format!(
            "{field} must stay under `{}`",
            root.display()
        )));
    }
    Ok(())
}

fn validate_service_path(field: &str, value: &str) -> Result<(), DeploymentStackCheckError> {
    if !value.starts_with('/') {
        return Err(DeploymentStackCheckError::Contract(format!(
            "{field} must start with `/`, got `{value}`"
        )));
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: afterburner deploy stack-check --target-profile PATH --stack-profile PATH [--out PATH]"
}

#[cfg(test)]
mod tests {
    use super::{Args, parse_args};
    use std::path::PathBuf;

    #[test]
    fn parse_args_accepts_required_flags() {
        let args = vec![
            "--target-profile".to_string(),
            "fixtures/deployment_target_profile.example.json".to_string(),
            "--stack-profile".to_string(),
            "fixtures/deployment_stack_profile.example.json".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse deployment stack args");
        assert_eq!(
            parsed,
            Args {
                target_profile: PathBuf::from("fixtures/deployment_target_profile.example.json"),
                stack_profile: PathBuf::from("fixtures/deployment_stack_profile.example.json"),
                out_path: None,
            }
        );
    }
}
