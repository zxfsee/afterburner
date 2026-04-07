use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_evidence_bundle_reconciliation.json";

#[derive(Debug)]
enum DeploymentStackLaunchBundleReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchBundleReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchBundleReconcileError {}

impl From<std::io::Error> for DeploymentStackLaunchBundleReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchBundleReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch evidence bundle reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    receipt: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchBundleReconcileError>
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
    let bundle = load_json_object(
        args.bundle.as_path(),
        "deployment stack launch evidence bundle",
    )?;

    let desired = json!({
        "plan_path": plan_path.display().to_string(),
        "receipt_path": args.receipt.display().to_string(),
        "profile_name": read_string(&plan, "profile_name", plan_path.as_path(), "deployment stack launch plan")?,
        "deploy_hostname": read_string(&plan, "deploy_hostname", plan_path.as_path(), "deployment stack launch plan")?,
        "deployable_unit": read_string(&plan, "deployable_unit", plan_path.as_path(), "deployment stack launch plan")?,
        "working_directory": read_string(&plan, "working_directory", plan_path.as_path(), "deployment stack launch plan")?,
        "service_entrypoint": read_string(&plan, "service_entrypoint", plan_path.as_path(), "deployment stack launch plan")?,
        "rollout_entrypoint": read_string(&plan, "rollout_entrypoint", plan_path.as_path(), "deployment stack launch plan")?,
        "launch_env": read_object(&receipt, "launch_env", args.receipt.as_path(), "deployment stack launch receipt")?,
        "launched_at_unix_ms": read_u64(&receipt, "launched_at_unix_ms", args.receipt.as_path(), "deployment stack launch receipt")?,
        "evidence_count": Value::from(2_u64),
    });
    let current = json!({
        "plan_path": read_string(&bundle, "plan_path", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "receipt_path": read_string(&bundle, "receipt_path", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "profile_name": read_string(&bundle, "profile_name", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "deploy_hostname": read_string(&bundle, "deploy_hostname", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "deployable_unit": read_string(&bundle, "deployable_unit", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "working_directory": read_string(&bundle, "working_directory", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "service_entrypoint": read_string(&bundle, "service_entrypoint", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "rollout_entrypoint": read_string(&bundle, "rollout_entrypoint", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "launch_env": read_object(&bundle, "launch_env", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "launched_at_unix_ms": read_u64(&bundle, "launched_at_unix_ms", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
    });
    let reconciliation_status = if desired == current {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "bundle_path": args.bundle.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_evidence_bundle_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchBundleReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut bundle = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                receipt = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--receipt",
                )?))
            }
            "--bundle" => {
                bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--bundle",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
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
                return Err(DeploymentStackLaunchBundleReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy reconcile-launch-bundle: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DeploymentStackLaunchBundleReconcileError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        bundle: bundle.ok_or_else(|| {
            DeploymentStackLaunchBundleReconcileError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentStackLaunchBundleReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchBundleReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchBundleReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(DeploymentStackLaunchBundleReconcileError);

fn usage() -> &'static str {
    "usage: afterburner debug deploy launch bundle reconcile --receipt PATH --bundle PATH [--out PATH]"
}
