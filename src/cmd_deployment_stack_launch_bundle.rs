use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEPLOYMENT_STACK_LAUNCH_EVIDENCE_BUNDLE_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_evidence_bundle.json";

#[derive(Debug)]
enum DeploymentStackLaunchBundleError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchBundleError {}

impl From<std::io::Error> for DeploymentStackLaunchBundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchBundleError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch evidence bundle json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    receipt: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchBundleError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "deployment stack launch receipt")?;
    let plan_path = PathBuf::from(read_string(
        &receipt,
        "plan_path",
        args.receipt.as_path(),
        "deployment stack launch receipt",
    )?);
    let plan = load_json_object(plan_path.as_path(), "deployment stack launch plan")?;

    for key in [
        "profile_name",
        "deploy_hostname",
        "deployable_unit",
        "service_entrypoint",
    ] {
        let plan_value = read_string(
            &plan,
            key,
            plan_path.as_path(),
            "deployment stack launch plan",
        )?;
        let receipt_value = read_string(
            &receipt,
            key,
            args.receipt.as_path(),
            "deployment stack launch receipt",
        )?;
        if plan_value != receipt_value {
            return Err(DeploymentStackLaunchBundleError::Parse(format!(
                "deployment stack launch receipt `{}` does not match launch plan `{}` for `{key}`",
                args.receipt.display(),
                plan_path.display()
            )));
        }
    }

    let launch_env = receipt
        .get("launch_env")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| {
            DeploymentStackLaunchBundleError::Parse(format!(
                "deployment stack launch receipt `{}` missing object field `launch_env`",
                args.receipt.display()
            ))
        })?;

    let bundle = json!({
        "schema_version": "1",
        "plan_path": plan_path.display().to_string(),
        "receipt_path": args.receipt.display().to_string(),
        "profile_name": read_string(&plan, "profile_name", plan_path.as_path(), "deployment stack launch plan")?,
        "deploy_hostname": read_string(&plan, "deploy_hostname", plan_path.as_path(), "deployment stack launch plan")?,
        "deployable_unit": read_string(&plan, "deployable_unit", plan_path.as_path(), "deployment stack launch plan")?,
        "working_directory": read_string(&plan, "working_directory", plan_path.as_path(), "deployment stack launch plan")?,
        "service_entrypoint": read_string(&plan, "service_entrypoint", plan_path.as_path(), "deployment stack launch plan")?,
        "rollout_entrypoint": read_string(&plan, "rollout_entrypoint", plan_path.as_path(), "deployment stack launch plan")?,
        "launch_env": Value::Object(launch_env),
        "launched_at_unix_ms": read_u64(&receipt, "launched_at_unix_ms", args.receipt.as_path(), "deployment stack launch receipt")?,
        "evidence_count": 2,
        "evidence_sources": [
            {
                "artifact_path": plan_path.display().to_string(),
                "artifact_role": "deployment_stack_launch_plan"
            },
            {
                "artifact_path": args.receipt.display().to_string(),
                "artifact_role": "deployment_stack_launch_receipt"
            }
        ]
    });

    write_json_value(&args.out_path, &bundle)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_evidence_bundle_written",
        "bundle_path",
        &args.out_path,
        "bundle",
        bundle,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchBundleError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_STACK_LAUNCH_EVIDENCE_BUNDLE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                let value: String = parse_value(&mut args, "--receipt")?;
                receipt = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentStackLaunchBundleError::InvalidArg(format!(
                    "unknown argument for deploy stack-launch-bundle: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DeploymentStackLaunchBundleError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DeploymentStackLaunchBundleError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchBundleError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchBundleError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(DeploymentStackLaunchBundleError);

fn usage() -> &'static str {
    "usage: afterburner debug deploy launch bundle --receipt PATH [--out PATH]"
}
