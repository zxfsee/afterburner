use std::any::Any;
use std::path::PathBuf;

use afterburner::model::ModelConfig;
use afterburner::observability::emit_event;
use afterburner::text_pretrain::{TextTrainingConfig, train_text};
use afterburner::train;
use burn::prelude::*;
use burn::{
    backend::ndarray::NdArray,
    backend::wgpu::{Metal, Wgpu},
    tensor::backend::AutodiffBackend,
};
use burn_autodiff::Autodiff;
use serde_json::json;

type GpuBackend = Wgpu<f32, i32>;
type MetalBackend = Metal<f32, i32>;
type CpuBackend = NdArray<f32>;
type GpuAutodiff = Autodiff<GpuBackend>;
type MetalAutodiff = Autodiff<MetalBackend>;
type CpuAutodiff = Autodiff<CpuBackend>;

pub fn run<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args: Vec<String> = args.collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return 0;
    }

    let backend = std::env::var("BACKEND")
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_else(|_| "wgpu".to_string());

    let parsed = match parse_args(args.into_iter()) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("{err}");
            return 2;
        }
    };

    let mnist_config = training_config_from_env(&parsed);
    let text_config = text_training_config_from_env(&parsed);

    if backend == "cpu" {
        let device = <CpuBackend as Backend>::Device::default();
        return run_task::<CpuAutodiff>(&parsed, mnist_config, text_config, device);
    }

    let parsed_for_fallback = parsed.clone();
    let gpu_result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match backend.as_str() {
            "metal" => {
                let device = <MetalBackend as Backend>::Device::default();
                run_task::<MetalAutodiff>(
                    &parsed,
                    mnist_config.clone(),
                    text_config.clone(),
                    device,
                )
            }
            _ => {
                let device = <GpuBackend as Backend>::Device::default();
                run_task::<GpuAutodiff>(&parsed, mnist_config.clone(), text_config.clone(), device)
            }
        }));

    match gpu_result {
        Ok(code) => code,
        Err(payload) => {
            if is_missing_gpu_adapter_panic(payload.as_ref()) {
                emit_event(
                    "warn",
                    "train_cli",
                    "backend_fallback",
                    json!({"from":backend,"to":"cpu","reason":"no_adapter"}),
                );
                let cpu_config = training_config_from_env(&parsed_for_fallback);
                let cpu_text_config = text_training_config_from_env(&parsed_for_fallback);
                let device = <CpuBackend as Backend>::Device::default();
                return run_task::<CpuAutodiff>(
                    &parsed_for_fallback,
                    cpu_config,
                    cpu_text_config,
                    device,
                );
            }

            std::panic::resume_unwind(payload);
        }
    }
}

fn run_task<B: AutodiffBackend>(
    args: &TrainArgs,
    mnist_config: train::TrainingConfig,
    text_config: Option<TextTrainingConfig>,
    device: B::Device,
) -> i32 {
    match args.task {
        TrainTask::Mnist => {
            train::train::<B>(mnist_config, device);
            0
        }
        TrainTask::Text => {
            let Some(config) = text_config else {
                eprintln!("missing text training configuration\n{}", usage());
                return 2;
            };
            match train_text::<B>(config, device) {
                Ok(()) => 0,
                Err(err) => {
                    eprintln!("{err}");
                    2
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrainArgs {
    task: TrainTask,
    batch_size: Option<usize>,
    num_workers: Option<usize>,
    num_epochs: Option<usize>,
    dataset_manifest: Option<PathBuf>,
    tokenizer_profile: Option<PathBuf>,
    token_cache: Option<PathBuf>,
    resume_epoch: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrainTask {
    Mnist,
    Text,
}

fn training_config_from_env(args: &TrainArgs) -> train::TrainingConfig {
    let artifact_dir = std::env::var("ARTIFACTS_DIR").ok();
    let artifact_version = std::env::var("ARTIFACT_VERSION").ok();

    let mut config = train::TrainingConfig::new(ModelConfig::new(10));
    if let Some(dir) = artifact_dir {
        config.artifacts_dir = dir;
    }
    if let Some(version) = artifact_version {
        config.artifact_version = version;
    }
    apply_cli_overrides(&mut config, args);

    config
}

fn text_training_config_from_env(args: &TrainArgs) -> Option<TextTrainingConfig> {
    if args.task != TrainTask::Text {
        return None;
    }

    Some(TextTrainingConfig {
        batch_size: args.batch_size.unwrap_or(2),
        num_workers: args.num_workers.unwrap_or(0),
        num_epochs: args.num_epochs.unwrap_or(1),
        learning_rate: 1e-3,
        seed: 42,
        artifacts_dir: std::env::var("ARTIFACTS_DIR").unwrap_or_else(|_| "artifacts".to_string()),
        artifact_version: std::env::var("ARTIFACT_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
        dataset_manifest_path: args
            .dataset_manifest
            .clone()
            .expect("dataset manifest required for text task"),
        tokenizer_profile_path: args
            .tokenizer_profile
            .clone()
            .expect("tokenizer profile required for text task"),
        token_cache_path: args
            .token_cache
            .clone()
            .expect("token cache required for text task"),
        resume_epoch: args.resume_epoch,
    })
}

fn apply_cli_overrides(config: &mut train::TrainingConfig, args: &TrainArgs) {
    if let Some(batch_size) = args.batch_size {
        config.batch_size = batch_size;
    }
    if let Some(num_workers) = args.num_workers {
        config.num_workers = num_workers;
    }
    if let Some(num_epochs) = args.num_epochs {
        config.num_epochs = num_epochs;
    }
}

fn parse_args<I>(args: I) -> Result<TrainArgs, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut parsed = TrainArgs {
        task: TrainTask::Mnist,
        batch_size: None,
        num_workers: None,
        num_epochs: None,
        dataset_manifest: None,
        tokenizer_profile: None,
        token_cache: None,
        resume_epoch: None,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--task" => parsed.task = parse_task(&mut args)?,
            "--batch-size" => parsed.batch_size = Some(parse_value(&mut args, "--batch-size")?),
            "--num-workers" => parsed.num_workers = Some(parse_value(&mut args, "--num-workers")?),
            "--num-epochs" => parsed.num_epochs = Some(parse_value(&mut args, "--num-epochs")?),
            "--dataset-manifest" => {
                parsed.dataset_manifest = Some(PathBuf::from(parse_string_value(
                    &mut args,
                    "--dataset-manifest",
                )?))
            }
            "--tokenizer-profile" => {
                parsed.tokenizer_profile = Some(PathBuf::from(parse_string_value(
                    &mut args,
                    "--tokenizer-profile",
                )?))
            }
            "--token-cache" => {
                parsed.token_cache = Some(PathBuf::from(parse_string_value(
                    &mut args,
                    "--token-cache",
                )?))
            }
            "--resume-epoch" => {
                parsed.resume_epoch = Some(parse_value(&mut args, "--resume-epoch")?)
            }
            _ if arg.starts_with("--task=") => {
                parsed.task = parse_task_value(arg.trim_start_matches("--task="))?
            }
            _ if arg.starts_with("--batch-size=") => {
                parsed.batch_size = Some(parse_inline_value(
                    arg.trim_start_matches("--batch-size="),
                    "--batch-size",
                )?)
            }
            _ if arg.starts_with("--num-workers=") => {
                parsed.num_workers = Some(parse_inline_value(
                    arg.trim_start_matches("--num-workers="),
                    "--num-workers",
                )?)
            }
            _ if arg.starts_with("--num-epochs=") => {
                parsed.num_epochs = Some(parse_inline_value(
                    arg.trim_start_matches("--num-epochs="),
                    "--num-epochs",
                )?)
            }
            _ => return Err(format!("unknown argument for train: {arg}\n{}", usage())),
        }
    }

    if matches!(parsed.batch_size, Some(0)) {
        return Err(format!("--batch-size must be > 0\n{}", usage()));
    }
    if matches!(parsed.num_epochs, Some(0)) {
        return Err(format!("--num-epochs must be > 0\n{}", usage()));
    }
    if matches!(parsed.resume_epoch, Some(0)) {
        return Err(format!("--resume-epoch must be > 0\n{}", usage()));
    }
    if parsed.task == TrainTask::Text
        && (parsed.dataset_manifest.is_none()
            || parsed.tokenizer_profile.is_none()
            || parsed.token_cache.is_none())
    {
        return Err(format!(
            "--task text requires --dataset-manifest, --tokenizer-profile, and --token-cache\n{}",
            usage()
        ));
    }

    Ok(parsed)
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args
        .next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))?;
    parse_inline_value(value.as_str(), flag)
}

fn parse_inline_value<T>(value: &str, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value
        .parse::<T>()
        .map_err(|_| format!("invalid value for {flag}: {value}\n{}", usage()))
}

fn parse_string_value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))
}

fn parse_task<I>(args: &mut I) -> Result<TrainTask, String>
where
    I: Iterator<Item = String>,
{
    parse_string_value(args, "--task").and_then(|value| parse_task_value(value.as_str()))
}

fn parse_task_value(value: &str) -> Result<TrainTask, String> {
    match value {
        "mnist" => Ok(TrainTask::Mnist),
        "text" => Ok(TrainTask::Text),
        _ => Err(format!(
            "invalid value for --task: {value} (expected `mnist` or `text`)\n{}",
            usage()
        )),
    }
}

fn is_missing_gpu_adapter_panic(payload: &(dyn Any + Send)) -> bool {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return message.contains("No possible adapter available for backend");
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.contains("No possible adapter available for backend");
    }
    false
}

fn usage() -> &'static str {
    "usage: afterburner train [--task mnist|text] [--batch-size N] [--num-workers N] [--num-epochs N] [--dataset-manifest PATH --tokenizer-profile PATH --token-cache PATH [--resume-epoch N]]"
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{TrainArgs, TrainTask, parse_args, training_config_from_env};

    #[test]
    fn parse_args_accepts_scalability_controls() {
        let args = vec![
            "--batch-size".to_string(),
            "32".to_string(),
            "--num-epochs".to_string(),
            "1".to_string(),
            "--num-workers=4".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse args");
        assert_eq!(
            parsed,
            TrainArgs {
                task: TrainTask::Mnist,
                batch_size: Some(32),
                num_workers: Some(4),
                num_epochs: Some(1),
                dataset_manifest: None,
                tokenizer_profile: None,
                token_cache: None,
                resume_epoch: None,
            }
        );
    }

    #[test]
    fn parse_args_rejects_zero_batch_size() {
        let args = vec!["--batch-size".to_string(), "0".to_string()];
        let err = parse_args(args.into_iter()).expect_err("zero batch size must fail");
        assert!(err.contains("--batch-size must be > 0"));
    }

    #[test]
    fn parse_args_rejects_zero_num_epochs() {
        let args = vec!["--num-epochs".to_string(), "0".to_string()];
        let err = parse_args(args.into_iter()).expect_err("zero num epochs must fail");
        assert!(err.contains("--num-epochs must be > 0"));
    }

    #[test]
    fn training_config_from_env_applies_epoch_override() {
        let config = training_config_from_env(&TrainArgs {
            task: TrainTask::Mnist,
            batch_size: None,
            num_workers: None,
            num_epochs: Some(1),
            dataset_manifest: None,
            tokenizer_profile: None,
            token_cache: None,
            resume_epoch: None,
        });
        assert_eq!(config.num_epochs, 1);
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let args = vec!["--bogus".to_string()];
        let err = parse_args(args.into_iter()).expect_err("unknown flag must fail");
        assert!(err.contains("unknown argument for train: --bogus"));
    }

    #[test]
    fn parse_args_accepts_text_task() {
        let args = vec![
            "--task".to_string(),
            "text".to_string(),
            "--dataset-manifest".to_string(),
            "fixtures/pretraining_dataset_manifest.example.json".to_string(),
            "--tokenizer-profile".to_string(),
            "fixtures/text_tokenizer_packing_profile.example.json".to_string(),
            "--token-cache".to_string(),
            "fixtures/text_token_cache.example.json".to_string(),
            "--resume-epoch".to_string(),
            "1".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse text args");
        assert_eq!(parsed.task, TrainTask::Text);
        assert_eq!(parsed.resume_epoch, Some(1));
        assert_eq!(
            parsed.dataset_manifest.as_deref(),
            Some(Path::new(
                "fixtures/pretraining_dataset_manifest.example.json"
            ))
        );
    }
}
