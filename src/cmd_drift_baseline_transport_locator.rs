use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::json;

const INFER_OUTPUT_DRIFT_BASELINE_TRANSPORT_LOCATOR_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_transport_locator.json";

#[derive(Debug)]
enum DriftBaselineTransportLocatorError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselineTransportLocatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselineTransportLocatorError {}

impl From<std::io::Error> for DriftBaselineTransportLocatorError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselineTransportLocatorError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let handoff = load_json_object(
        args.handoff.as_path(),
        "infer output drift baseline handoff",
    )?;

    let locator = json!({
        "schema_version": "1",
        "handoff_path": args.handoff.display().to_string(),
        "artifact": read_string(&handoff, "artifact", args.handoff.as_path())?,
        "artifact_version": read_string(&handoff, "artifact_version", args.handoff.as_path())?,
        "policy_profile": read_string(&handoff, "policy_profile", args.handoff.as_path())?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&locator).map_err(|err| {
        DriftBaselineTransportLocatorError::Parse(format!(
            "serialize baseline transport locator json: {err}"
        ))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_transport_locator_written",
        json!({
            "locator_path": args.out_path.display().to_string(),
            "locator": locator,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselineTransportLocatorError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut handoff = None::<PathBuf>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_TRANSPORT_LOCATOR_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--handoff" => {
                let value: String = parse_value(&mut args, "--handoff")?;
                handoff = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
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
                return Err(DriftBaselineTransportLocatorError::InvalidArg(format!(
                    "unknown argument for drift point-baseline-transport-locator: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        handoff: handoff.ok_or_else(|| {
            DriftBaselineTransportLocatorError::InvalidArg(format!(
                "missing value for --handoff\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselineTransportLocatorError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselineTransportLocatorError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselineTransportLocatorError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_kind_helpers!(
    DriftBaselineTransportLocatorError,
    "drift baseline handoff"
);

fn usage() -> &'static str {
    "usage: afterburner debug drift baseline transport point --handoff PATH [--out PATH]"
}
