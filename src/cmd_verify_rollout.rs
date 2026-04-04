use std::path::{Path, PathBuf};
use std::process::Command;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_EVAL_OUT: &str = "artifacts/eval/mnist_eval_summary.json";
const DEFAULT_RECORD_OUT: &str = "artifacts/deploy/rollout_verify.json";

const DEFAULT_SEED: u64 = 42;
const DEFAULT_BATCH_SIZE: u64 = 128;
const DEFAULT_MAX_BATCHES: u64 = 8;
const DEFAULT_MIN_ACCURACY: f64 = 0.98925781;

#[derive(Debug)]
enum VerifyRolloutError {
    InvalidArg(String),
    Io(std::io::Error),
    Json(JsonArtifactError),
    Process(String),
    Parse(String),
}

impl std::fmt::Display for VerifyRolloutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "{err}"),
            Self::Process(msg) => write!(f, "{msg}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for VerifyRolloutError {}

impl From<std::io::Error> for VerifyRolloutError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for VerifyRolloutError {
    fn from(value: JsonArtifactError) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Args {
    candidate_artifact: PathBuf,
    eval_out: PathBuf,
    out_record: PathBuf,
    repo_root: PathBuf,
    seed: u64,
    batch_size: u64,
    max_batches: u64,
    min_accuracy: f64,
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

fn run_inner<I>(args: I) -> Result<PathBuf, VerifyRolloutError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;

    let infer_stdout = run_afterburner(args.repo_root.as_path(), &["infer".to_string()])?;
    let infer_output: Value = serde_json::from_str(&infer_stdout)
        .map_err(|err| VerifyRolloutError::Parse(format!("parse infer stdout json: {err}")))?;

    run_afterburner(
        args.repo_root.as_path(),
        &[
            "eval".to_string(),
            "--artifact".to_string(),
            args.candidate_artifact.display().to_string(),
            "--seed".to_string(),
            args.seed.to_string(),
            "--batch-size".to_string(),
            args.batch_size.to_string(),
            "--max-batches".to_string(),
            args.max_batches.to_string(),
            "--min-accuracy".to_string(),
            args.min_accuracy.to_string(),
            "--out".to_string(),
            args.eval_out.display().to_string(),
        ],
    )?;

    let payload = json!({
        "schema_version": "1",
        "candidate_artifact": args.candidate_artifact.display().to_string(),
        "infer_output": infer_output,
        "eval_summary_path": args.eval_out.display().to_string(),
    });
    write_json_value(&args.out_record, &payload)?;
    emit_json_artifact_written(
        "verify_cli",
        "rollout_verify_written",
        "record_path",
        &args.out_record,
        "record",
        payload,
    );
    Ok(args.out_record)
}

fn parse_args<I>(args: I) -> Result<Args, VerifyRolloutError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut candidate_artifact = None::<PathBuf>;
    let mut eval_out = PathBuf::from(DEFAULT_EVAL_OUT);
    let mut out_record = PathBuf::from(DEFAULT_RECORD_OUT);
    let mut repo_root = PathBuf::from(".");
    let mut seed = DEFAULT_SEED;
    let mut batch_size = DEFAULT_BATCH_SIZE;
    let mut max_batches = DEFAULT_MAX_BATCHES;
    let mut min_accuracy = DEFAULT_MIN_ACCURACY;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => {
                candidate_artifact = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--artifact",
                )?))
            }
            "--eval-out" => {
                eval_out = PathBuf::from(parse_value::<String, _>(&mut args, "--eval-out")?)
            }
            "--out-record" => {
                out_record = PathBuf::from(parse_value::<String, _>(&mut args, "--out-record")?)
            }
            "--repo-root" => {
                repo_root = PathBuf::from(parse_value::<String, _>(&mut args, "--repo-root")?)
            }
            "--seed" => seed = parse_value(&mut args, "--seed")?,
            "--batch-size" => batch_size = parse_value(&mut args, "--batch-size")?,
            "--max-batches" => max_batches = parse_value(&mut args, "--max-batches")?,
            "--min-accuracy" => min_accuracy = parse_value(&mut args, "--min-accuracy")?,
            _ if arg.starts_with("--artifact=") => {
                candidate_artifact = Some(PathBuf::from(
                    arg.trim_start_matches("--artifact=").to_string(),
                ))
            }
            _ if arg.starts_with("--eval-out=") => {
                eval_out = PathBuf::from(arg.trim_start_matches("--eval-out=").to_string())
            }
            _ if arg.starts_with("--out-record=") => {
                out_record = PathBuf::from(arg.trim_start_matches("--out-record=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--seed=") => {
                seed = arg
                    .trim_start_matches("--seed=")
                    .parse::<u64>()
                    .map_err(|_| invalid_value("--seed"))?
            }
            _ if arg.starts_with("--batch-size=") => {
                batch_size = arg
                    .trim_start_matches("--batch-size=")
                    .parse::<u64>()
                    .map_err(|_| invalid_value("--batch-size"))?
            }
            _ if arg.starts_with("--max-batches=") => {
                max_batches = arg
                    .trim_start_matches("--max-batches=")
                    .parse::<u64>()
                    .map_err(|_| invalid_value("--max-batches"))?
            }
            _ if arg.starts_with("--min-accuracy=") => {
                min_accuracy = arg
                    .trim_start_matches("--min-accuracy=")
                    .parse::<f64>()
                    .map_err(|_| invalid_value("--min-accuracy"))?
            }
            _ => {
                return Err(VerifyRolloutError::InvalidArg(format!(
                    "unknown argument for verify rollout: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        candidate_artifact: candidate_artifact.ok_or_else(|| {
            VerifyRolloutError::InvalidArg(format!("missing value for --artifact\n{}", usage()))
        })?,
        eval_out,
        out_record,
        repo_root,
        seed,
        batch_size,
        max_batches,
        min_accuracy,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, VerifyRolloutError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        VerifyRolloutError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        VerifyRolloutError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn invalid_value(flag: &str) -> VerifyRolloutError {
    VerifyRolloutError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
}

fn run_afterburner(repo_root: &Path, args: &[String]) -> Result<String, VerifyRolloutError> {
    let exe = std::env::current_exe().map_err(VerifyRolloutError::Io)?;
    let output = Command::new(exe)
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if output.status.success() {
        return String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|err| {
                VerifyRolloutError::Parse(format!("subcommand stdout is not utf8: {err}"))
            });
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() { stderr } else { stdout };
    Err(VerifyRolloutError::Process(format!(
        "afterburner {} failed: {detail}",
        args.join(" ")
    )))
}

fn usage() -> &'static str {
    "usage: afterburner verify rollout --artifact PATH [--eval-out PATH] [--out-record PATH] [--repo-root PATH] [--seed N] [--batch-size N] [--max-batches N] [--min-accuracy F64]"
}
