use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_locator_reconciliation.json";

#[derive(Debug)]
enum DeploymentStackLaunchLocatorReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchLocatorReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchLocatorReconcileError {}

impl From<std::io::Error> for DeploymentStackLaunchLocatorReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchLocatorReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch locator reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    plan: PathBuf,
    pointer: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchLocatorReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let plan = load_json_object(args.plan.as_path(), "deployment stack launch plan")?;
    let pointer = load_json_object(
        args.pointer.as_path(),
        "deployment stack launch locator pointer",
    )?;

    let desired = json!({
        "profile_name": read_string(&plan, "profile_name", args.plan.as_path(), "deployment stack launch plan")?,
        "deploy_hostname": read_string(&plan, "deploy_hostname", args.plan.as_path(), "deployment stack launch plan")?,
        "rollout_entrypoint": read_string(&plan, "rollout_entrypoint", args.plan.as_path(), "deployment stack launch plan")?,
    });
    let current = json!({
        "locator_path": read_string(&pointer, "locator_path", args.pointer.as_path(), "deployment stack launch locator pointer")?,
        "profile_name": read_string(&pointer, "profile_name", args.pointer.as_path(), "deployment stack launch locator pointer")?,
        "deploy_hostname": read_string(&pointer, "deploy_hostname", args.pointer.as_path(), "deployment stack launch locator pointer")?,
        "rollout_entrypoint": read_string(&pointer, "rollout_entrypoint", args.pointer.as_path(), "deployment stack launch locator pointer")?,
    });
    let reconciliation_status = if desired["profile_name"] == current["profile_name"]
        && desired["deploy_hostname"] == current["deploy_hostname"]
        && desired["rollout_entrypoint"] == current["rollout_entrypoint"]
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "plan_path": args.plan.display().to_string(),
        "pointer_path": args.pointer.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_locator_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchLocatorReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut plan = None::<PathBuf>;
    let mut pointer = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--plan" => {
                plan = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--plan",
                )?))
            }
            "--pointer" => {
                pointer = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--pointer",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--plan=") => {
                plan = Some(PathBuf::from(arg.trim_start_matches("--plan=").to_string()))
            }
            _ if arg.starts_with("--pointer=") => {
                pointer = Some(PathBuf::from(
                    arg.trim_start_matches("--pointer=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentStackLaunchLocatorReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy reconcile-launch-locator: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        plan: plan.ok_or_else(|| {
            DeploymentStackLaunchLocatorReconcileError::InvalidArg(format!(
                "missing value for --plan\n{}",
                usage()
            ))
        })?,
        pointer: pointer.ok_or_else(|| {
            DeploymentStackLaunchLocatorReconcileError::InvalidArg(format!(
                "missing value for --pointer\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentStackLaunchLocatorReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchLocatorReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchLocatorReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(DeploymentStackLaunchLocatorReconcileError);

fn usage() -> &'static str {
    "usage: afterburner debug deploy launch locator reconcile --plan PATH --pointer PATH [--out PATH]"
}
