use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::json;

const DEPLOYMENT_STACK_LAUNCH_LOCATOR_POINTER_PATH: &str =
    "artifacts/deploy/deployment_stack_launch_locator_pointer.json";

#[derive(Debug)]
enum DeploymentStackLaunchLocatorPointerError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DeploymentStackLaunchLocatorPointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DeploymentStackLaunchLocatorPointerError {}

impl From<std::io::Error> for DeploymentStackLaunchLocatorPointerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    locator: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DeploymentStackLaunchLocatorPointerError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let locator = load_json_object(
        args.locator.as_path(),
        "deployment stack launch transport locator",
    )?;

    let pointer = json!({
        "schema_version": "1",
        "locator_path": args.locator.display().to_string(),
        "profile_name": read_string(&locator, "profile_name", args.locator.as_path())?,
        "deploy_hostname": read_string(&locator, "deploy_hostname", args.locator.as_path())?,
        "rollout_entrypoint": read_string(&locator, "rollout_entrypoint", args.locator.as_path())?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&pointer).map_err(|err| {
        DeploymentStackLaunchLocatorPointerError::Parse(format!(
            "serialize deployment stack launch locator pointer json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "deployment_stack_launch_locator_pointer_written",
        json!({
            "pointer_path": args.out_path.display().to_string(),
            "pointer": pointer,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DeploymentStackLaunchLocatorPointerError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut locator = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEPLOYMENT_STACK_LAUNCH_LOCATOR_POINTER_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--locator" => {
                let value: String = parse_value(&mut args, "--locator")?;
                locator = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--locator=") => {
                locator = Some(PathBuf::from(
                    arg.trim_start_matches("--locator=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DeploymentStackLaunchLocatorPointerError::InvalidArg(
                    format!(
                        "unknown argument for deploy point-launch-locator: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        locator: locator.ok_or_else(|| {
            DeploymentStackLaunchLocatorPointerError::InvalidArg(format!(
                "missing value for --locator\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DeploymentStackLaunchLocatorPointerError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeploymentStackLaunchLocatorPointerError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DeploymentStackLaunchLocatorPointerError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_kind_helpers!(
    DeploymentStackLaunchLocatorPointerError,
    "deployment stack launch transport locator"
);

fn usage() -> &'static str {
    "usage: afterburner deploy point-launch-locator --locator PATH [--out PATH]"
}
