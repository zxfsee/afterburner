use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::json;

const INFER_OUTPUT_DRIFT_BASELINE_POINTER_PATH: &str =
    "artifacts/eval/infer_output_drift_baseline_pointer.json";

#[derive(Debug)]
enum DriftBaselinePointerError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DriftBaselinePointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DriftBaselinePointerError {}

impl From<std::io::Error> for DriftBaselinePointerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    approval: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DriftBaselinePointerError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let approval = load_json_object(
        args.approval.as_path(),
        "infer output drift baseline approval",
    )?;

    let pointer = json!({
        "schema_version": "1",
        "approval_path": args.approval.display().to_string(),
        "artifact": read_string(&approval, "artifact", args.approval.as_path())?,
        "artifact_version": read_string(&approval, "artifact_version", args.approval.as_path())?,
        "policy_profile": read_string(&approval, "policy_profile", args.approval.as_path())?,
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&pointer).map_err(|err| {
        DriftBaselinePointerError::Parse(format!("serialize baseline pointer json: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "infer_cli",
        "infer_output_drift_baseline_pointer_written",
        json!({
            "pointer_path": args.out_path.display().to_string(),
            "pointer": pointer,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DriftBaselinePointerError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut approval = None::<PathBuf>;
    let mut out_path = PathBuf::from(INFER_OUTPUT_DRIFT_BASELINE_POINTER_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--approval" => {
                let value: String = parse_value(&mut args, "--approval")?;
                approval = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--approval=") => {
                approval = Some(PathBuf::from(
                    arg.trim_start_matches("--approval=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DriftBaselinePointerError::InvalidArg(format!(
                    "unknown argument for drift point-approved-baseline: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        approval: approval.ok_or_else(|| {
            DriftBaselinePointerError::InvalidArg(format!(
                "missing value for --approval\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DriftBaselinePointerError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DriftBaselinePointerError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DriftBaselinePointerError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

afterburner::define_command_input_json_kind_helpers!(
    DriftBaselinePointerError,
    "drift baseline transport locator"
);

fn usage() -> &'static str {
    "usage: afterburner drift point-approved-baseline --approval PATH [--out PATH]"
}
