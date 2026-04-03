use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEPLOYMENT_STACK_LAUNCH_RECEIPT_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_receipt.json";

#[derive(Debug)]
enum DeploymentStackLaunchReceiptError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchReceiptError {}

impl From<std::io::Error> for DeploymentStackLaunchReceiptError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeploymentStackLaunchReceiptError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize deployment stack launch receipt json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    plan: PathBuf,
    port: u16,
    launched_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchReceiptError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let plan = load_json_object(args.plan.as_path(), "deployment stack launch plan")?;
    let default_env = plan
        .get("default_env")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            DeploymentStackLaunchReceiptError::Parse(format!(
                "deployment stack launch plan `{}` missing object field `default_env`",
                args.plan.display()
            ))
        })?;

    let obs_path = default_env
        .get("AFTERBURNER_OBS_JSONL_PATH")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            DeploymentStackLaunchReceiptError::Parse(format!(
                "deployment stack launch plan `{}` missing default env `AFTERBURNER_OBS_JSONL_PATH`",
                args.plan.display()
            ))
        })?;

    let receipt = json!({
        "schema_version": "1",
        "plan_path": args.plan.display().to_string(),
        "profile_name": read_string(&plan, "profile_name", args.plan.as_path())?,
        "deploy_hostname": read_string(&plan, "deploy_hostname", args.plan.as_path())?,
        "deployable_unit": read_string(&plan, "deployable_unit", args.plan.as_path())?,
        "service_entrypoint": read_string(&plan, "service_entrypoint", args.plan.as_path())?,
        "launch_env": {
            "PORT": args.port.to_string(),
            "AFTERBURNER_OBS_JSONL_PATH": obs_path
        },
        "launched_at_unix_ms": args.launched_at_unix_ms
    });

    write_json_value(&args.out_path, &receipt)?;
    emit_json_artifact_written(
        "deploy_cli",
        "deployment_stack_launch_receipt_written",
        "receipt_path",
        &args.out_path,
        "receipt",
        receipt,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchReceiptError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut plan = None::<PathBuf>;
    let mut port = None::<u16>;
    let mut launched_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DEPLOYMENT_STACK_LAUNCH_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--plan" => {
                plan = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--plan",
                )?))
            }
            "--port" => port = Some(parse_value(&mut args, "--port")?),
            "--launched-at-unix-ms" => {
                launched_at_unix_ms = Some(parse_value(&mut args, "--launched-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--plan=") => {
                plan = Some(PathBuf::from(arg.trim_start_matches("--plan=").to_string()))
            }
            _ if arg.starts_with("--port=") => {
                port = Some(
                    arg.trim_start_matches("--port=")
                        .parse::<u16>()
                        .map_err(|_| {
                            DeploymentStackLaunchReceiptError::InvalidArg(format!(
                                "invalid value for --port\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--launched-at-unix-ms=") => {
                launched_at_unix_ms = Some(
                    arg.trim_start_matches("--launched-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DeploymentStackLaunchReceiptError::InvalidArg(format!(
                                "invalid value for --launched-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentStackLaunchReceiptError::InvalidArg(format!(
                    "unknown argument for deploy stack-launch-receipt: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        plan: plan.ok_or_else(|| {
            DeploymentStackLaunchReceiptError::InvalidArg(format!(
                "missing value for --plan\n{}",
                usage()
            ))
        })?,
        port: port.ok_or_else(|| {
            DeploymentStackLaunchReceiptError::InvalidArg(format!(
                "missing value for --port\n{}",
                usage()
            ))
        })?,
        launched_at_unix_ms: launched_at_unix_ms.ok_or_else(|| {
            DeploymentStackLaunchReceiptError::InvalidArg(format!(
                "missing value for --launched-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DeploymentStackLaunchReceiptError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchReceiptError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_kind_helpers!(
    DeploymentStackLaunchReceiptError,
    "deployment stack launch plan"
);

fn usage() -> &'static str {
    "usage: afterburner deploy stack-launch-receipt --plan PATH --port PORT --launched-at-unix-ms N [--out PATH]"
}
