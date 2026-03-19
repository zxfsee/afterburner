use std::fs;
use std::path::PathBuf;

use afterburner::observability::emit_event;
use serde_json::json;

const ARTIFACT_CLEANUP_POLICY_PATH: &str = "artifacts/deploy/artifact_cleanup_policy.json";

#[derive(Debug)]
enum CleanupPolicyError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for CleanupPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CleanupPolicyError {}

impl From<std::io::Error> for CleanupPolicyError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    profile_name: String,
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

fn run_inner<I>(args: I) -> Result<(), CleanupPolicyError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let policy = json!({
        "schema_version": "1",
        "profile_name": args.profile_name,
        "protected_categories": [
            "runtime_pointer",
            "active_inference_directory",
            "rollout_evidence",
            "train_contracts",
            "eval_state"
        ],
        "prune_candidate_categories": [
            "inactive_inference_directory",
            "profiling_artifacts"
        ]
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&policy).map_err(|err| {
        CleanupPolicyError::Parse(format!("serialize cleanup policy json: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "ops_cli",
        "artifact_cleanup_policy_written",
        json!({
            "policy_path": args.out_path.display().to_string(),
            "policy": policy,
        }),
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, CleanupPolicyError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut profile_name = None::<String>;
    let mut out_path = PathBuf::from(ARTIFACT_CLEANUP_POLICY_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--profile" => profile_name = Some(parse_value(&mut args, "--profile")?),
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--profile=") => {
                profile_name = Some(arg.trim_start_matches("--profile=").to_string())
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(CleanupPolicyError::InvalidArg(format!(
                    "unknown argument for cleanup-policy: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    let profile_name = profile_name.ok_or_else(|| {
        CleanupPolicyError::InvalidArg(format!("missing value for --profile\n{}", usage()))
    })?;
    if profile_name.trim().is_empty() {
        return Err(CleanupPolicyError::InvalidArg(format!(
            "--profile must be non-empty\n{}",
            usage()
        )));
    }

    Ok(Args {
        profile_name,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, CleanupPolicyError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        CleanupPolicyError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        CleanupPolicyError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner cleanup-policy --profile NAME [--out PATH]"
}
