use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_evidence_handoff_reconciliation.json";

#[derive(Debug)]
enum DeploymentStackLaunchHandoffReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchHandoffReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchHandoffReconcileError {}

impl From<std::io::Error> for DeploymentStackLaunchHandoffReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchHandoffReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch handoff reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    bundle: PathBuf,
    handoff: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchHandoffReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "deployment stack launch evidence bundle",
    )?;
    let handoff = load_json_object(
        args.handoff.as_path(),
        "deployment stack launch evidence handoff",
    )?;

    let desired = json!({
        "profile_name": read_string(&bundle, "profile_name", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "deploy_hostname": read_string(&bundle, "deploy_hostname", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "deployable_unit": read_string(&bundle, "deployable_unit", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "rollout_entrypoint": read_string(&bundle, "rollout_entrypoint", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "launched_at_unix_ms": read_u64(&bundle, "launched_at_unix_ms", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path(), "deployment stack launch evidence bundle")?,
    });
    let current = json!({
        "bundle_path": read_string(&handoff, "bundle_path", args.handoff.as_path(), "deployment stack launch evidence handoff")?,
        "profile_name": read_string(&handoff, "profile_name", args.handoff.as_path(), "deployment stack launch evidence handoff")?,
        "deploy_hostname": read_string(&handoff, "deploy_hostname", args.handoff.as_path(), "deployment stack launch evidence handoff")?,
        "deployable_unit": read_string(&handoff, "deployable_unit", args.handoff.as_path(), "deployment stack launch evidence handoff")?,
        "rollout_entrypoint": read_string(&handoff, "rollout_entrypoint", args.handoff.as_path(), "deployment stack launch evidence handoff")?,
        "launched_at_unix_ms": read_u64(&handoff, "launched_at_unix_ms", args.handoff.as_path(), "deployment stack launch evidence handoff")?,
        "evidence_count": read_u64(&handoff, "evidence_count", args.handoff.as_path(), "deployment stack launch evidence handoff")?,
    });
    let reconciliation_status = if desired["profile_name"] == current["profile_name"]
        && desired["deploy_hostname"] == current["deploy_hostname"]
        && desired["deployable_unit"] == current["deployable_unit"]
        && desired["rollout_entrypoint"] == current["rollout_entrypoint"]
        && desired["launched_at_unix_ms"] == current["launched_at_unix_ms"]
        && desired["evidence_count"] == current["evidence_count"]
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "bundle_path": args.bundle.display().to_string(),
        "handoff_path": args.handoff.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_evidence_handoff_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchHandoffReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut bundle = None::<PathBuf>;
    let mut handoff = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bundle" => {
                bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--bundle",
                )?))
            }
            "--handoff" => {
                handoff = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--handoff",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--bundle=") => {
                bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--handoff=") => {
                handoff = Some(PathBuf::from(
                    arg.trim_start_matches("--handoff=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentStackLaunchHandoffReconcileError::InvalidArg(
                    format!(
                        "unknown argument for deploy reconcile-launch-handoff: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        bundle: bundle.ok_or_else(|| {
            DeploymentStackLaunchHandoffReconcileError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        handoff: handoff.ok_or_else(|| {
            DeploymentStackLaunchHandoffReconcileError::InvalidArg(format!(
                "missing value for --handoff\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentStackLaunchHandoffReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchHandoffReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchHandoffReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(DeploymentStackLaunchHandoffReconcileError);

fn usage() -> &'static str {
    "usage: afterburner deploy reconcile-launch-handoff --bundle PATH --handoff PATH [--out PATH]"
}
