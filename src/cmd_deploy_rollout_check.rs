use std::path::{Path, PathBuf};
use std::process::Command;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_TARGET_PROFILE: &str = "fixtures/deployment_target_profile.example.json";
const DEFAULT_STACK_PROFILE: &str = "fixtures/deployment_stack_profile.example.json";
const DEFAULT_EVAL_OUT: &str = "artifacts/eval/mnist_eval_summary.json";
const DEFAULT_UPLOAD_OUT: &str = "artifacts/deploy/candidate_upload_request.json";
const DEFAULT_STACK_CHECK_OUT: &str = "artifacts/deploy/deployment_stack_check.json";
const DEFAULT_RECORD_OUT: &str = "artifacts/deploy/rollout_check.json";

const DEFAULT_SEED: u64 = 42;
const DEFAULT_BATCH_SIZE: u64 = 128;
const DEFAULT_MAX_BATCHES: u64 = 8;
const DEFAULT_MIN_ACCURACY: f64 = 0.98925781;

#[derive(Debug)]
enum DeployRolloutCheckError {
    InvalidArg(String),
    Io(std::io::Error),
    Json(JsonArtifactError),
    Process(String),
    Parse(String),
}

impl std::fmt::Display for DeployRolloutCheckError {
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

impl std::error::Error for DeployRolloutCheckError {}

impl From<std::io::Error> for DeployRolloutCheckError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DeployRolloutCheckError {
    fn from(value: JsonArtifactError) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Args {
    candidate_artifact: PathBuf,
    candidate_manifest: PathBuf,
    ownership: PathBuf,
    provider: String,
    destination: String,
    target_profile: PathBuf,
    stack_profile: PathBuf,
    eval_out: PathBuf,
    upload_out: PathBuf,
    stack_check_out: PathBuf,
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

fn run_inner<I>(args: I) -> Result<PathBuf, DeployRolloutCheckError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;

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

    run_afterburner(
        args.repo_root.as_path(),
        &[
            "deploy".to_string(),
            "upload".to_string(),
            "--manifest".to_string(),
            args.candidate_manifest.display().to_string(),
            "--ownership".to_string(),
            args.ownership.display().to_string(),
            "--provider".to_string(),
            args.provider.clone(),
            "--destination".to_string(),
            args.destination.clone(),
            "--out".to_string(),
            args.upload_out.display().to_string(),
        ],
    )?;

    run_afterburner(
        args.repo_root.as_path(),
        &[
            "deploy".to_string(),
            "stack-check".to_string(),
            "--target-profile".to_string(),
            args.target_profile.display().to_string(),
            "--stack-profile".to_string(),
            args.stack_profile.display().to_string(),
            "--out".to_string(),
            args.stack_check_out.display().to_string(),
        ],
    )?;

    let deploy_activate_drv = run_command_capture_stdout(
        args.repo_root.as_path(),
        "nix",
        &["eval", ".#checks.aarch64-darwin.deploy-activate.drvPath"],
    )?;
    let deploy_schema_drv = run_command_capture_stdout(
        args.repo_root.as_path(),
        "nix",
        &["eval", ".#checks.aarch64-darwin.deploy-schema.drvPath"],
    )?;

    let payload = json!({
        "schema_version": "1",
        "candidate_artifact": args.candidate_artifact.display().to_string(),
        "candidate_manifest": args.candidate_manifest.display().to_string(),
        "ownership_path": args.ownership.display().to_string(),
        "provider": args.provider,
        "destination": args.destination,
        "target_profile": args.target_profile.display().to_string(),
        "stack_profile": args.stack_profile.display().to_string(),
        "eval_summary_path": args.eval_out.display().to_string(),
        "upload_request_path": args.upload_out.display().to_string(),
        "deployment_stack_check_path": args.stack_check_out.display().to_string(),
        "deploy_activate_drv_path": deploy_activate_drv,
        "deploy_schema_drv_path": deploy_schema_drv,
    });

    write_json_value(&args.out_record, &payload)?;
    emit_json_artifact_written(
        "deploy_cli",
        "rollout_check_written",
        "record_path",
        &args.out_record,
        "record",
        payload,
    );
    Ok(args.out_record)
}

fn parse_args<I>(args: I) -> Result<Args, DeployRolloutCheckError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut candidate_artifact = None::<PathBuf>;
    let mut candidate_manifest = None::<PathBuf>;
    let mut ownership = None::<PathBuf>;
    let mut provider = None::<String>;
    let mut destination = None::<String>;
    let mut target_profile = PathBuf::from(DEFAULT_TARGET_PROFILE);
    let mut stack_profile = PathBuf::from(DEFAULT_STACK_PROFILE);
    let mut eval_out = PathBuf::from(DEFAULT_EVAL_OUT);
    let mut upload_out = PathBuf::from(DEFAULT_UPLOAD_OUT);
    let mut stack_check_out = PathBuf::from(DEFAULT_STACK_CHECK_OUT);
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
            "--manifest" => {
                candidate_manifest = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--manifest",
                )?))
            }
            "--ownership" => {
                ownership = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--ownership",
                )?))
            }
            "--provider" => provider = Some(parse_value(&mut args, "--provider")?),
            "--destination" => destination = Some(parse_value(&mut args, "--destination")?),
            "--target-profile" => {
                target_profile =
                    PathBuf::from(parse_value::<String, _>(&mut args, "--target-profile")?)
            }
            "--stack-profile" => {
                stack_profile =
                    PathBuf::from(parse_value::<String, _>(&mut args, "--stack-profile")?)
            }
            "--eval-out" => {
                eval_out = PathBuf::from(parse_value::<String, _>(&mut args, "--eval-out")?)
            }
            "--upload-out" => {
                upload_out = PathBuf::from(parse_value::<String, _>(&mut args, "--upload-out")?)
            }
            "--stack-check-out" => {
                stack_check_out =
                    PathBuf::from(parse_value::<String, _>(&mut args, "--stack-check-out")?)
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
            _ if arg.starts_with("--manifest=") => {
                candidate_manifest = Some(PathBuf::from(
                    arg.trim_start_matches("--manifest=").to_string(),
                ))
            }
            _ if arg.starts_with("--ownership=") => {
                ownership = Some(PathBuf::from(
                    arg.trim_start_matches("--ownership=").to_string(),
                ))
            }
            _ if arg.starts_with("--provider=") => {
                provider = Some(arg.trim_start_matches("--provider=").to_string())
            }
            _ if arg.starts_with("--destination=") => {
                destination = Some(arg.trim_start_matches("--destination=").to_string())
            }
            _ if arg.starts_with("--target-profile=") => {
                target_profile =
                    PathBuf::from(arg.trim_start_matches("--target-profile=").to_string())
            }
            _ if arg.starts_with("--stack-profile=") => {
                stack_profile =
                    PathBuf::from(arg.trim_start_matches("--stack-profile=").to_string())
            }
            _ if arg.starts_with("--eval-out=") => {
                eval_out = PathBuf::from(arg.trim_start_matches("--eval-out=").to_string())
            }
            _ if arg.starts_with("--upload-out=") => {
                upload_out = PathBuf::from(arg.trim_start_matches("--upload-out=").to_string())
            }
            _ if arg.starts_with("--stack-check-out=") => {
                stack_check_out =
                    PathBuf::from(arg.trim_start_matches("--stack-check-out=").to_string())
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
                return Err(DeployRolloutCheckError::InvalidArg(format!(
                    "unknown argument for deploy rollout-check: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        candidate_artifact: required_path(candidate_artifact, "--artifact")?,
        candidate_manifest: required_path(candidate_manifest, "--manifest")?,
        ownership: required_path(ownership, "--ownership")?,
        provider: required_string(provider, "--provider")?,
        destination: required_string(destination, "--destination")?,
        target_profile,
        stack_profile,
        eval_out,
        upload_out,
        stack_check_out,
        out_record,
        repo_root,
        seed,
        batch_size,
        max_batches,
        min_accuracy,
    })
}

fn required_path(value: Option<PathBuf>, flag: &str) -> Result<PathBuf, DeployRolloutCheckError> {
    value.ok_or_else(|| {
        DeployRolloutCheckError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })
}

fn required_string(value: Option<String>, flag: &str) -> Result<String, DeployRolloutCheckError> {
    value.filter(|v| !v.trim().is_empty()).ok_or_else(|| {
        DeployRolloutCheckError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DeployRolloutCheckError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DeployRolloutCheckError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        DeployRolloutCheckError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn invalid_value(flag: &str) -> DeployRolloutCheckError {
    DeployRolloutCheckError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
}

fn run_afterburner(repo_root: &Path, args: &[String]) -> Result<String, DeployRolloutCheckError> {
    let exe = std::env::current_exe().map_err(DeployRolloutCheckError::Io)?;
    let output = Command::new(exe)
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if output.status.success() {
        return String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|err| {
                DeployRolloutCheckError::Parse(format!("subcommand stdout is not utf8: {err}"))
            });
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() { stderr } else { stdout };
    Err(DeployRolloutCheckError::Process(format!(
        "afterburner {} failed: {detail}",
        args.join(" ")
    )))
}

fn run_command_capture_stdout(
    repo_root: &Path,
    program: &str,
    args: &[&str],
) -> Result<String, DeployRolloutCheckError> {
    let output = Command::new(program)
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if output.status.success() {
        return String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|err| {
                DeployRolloutCheckError::Parse(format!("{program} stdout is not utf8: {err}"))
            });
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(DeployRolloutCheckError::Process(format!(
        "{program} {} failed: {stderr}",
        args.join(" ")
    )))
}

fn usage() -> &'static str {
    "usage: afterburner deploy rollout-check --artifact PATH --manifest PATH --ownership PATH --provider NAME --destination VALUE [--target-profile PATH] [--stack-profile PATH] [--eval-out PATH] [--upload-out PATH] [--stack-check-out PATH] [--out-record PATH] [--repo-root PATH] [--seed N] [--batch-size N] [--max-batches N] [--min-accuracy F64]"
}
