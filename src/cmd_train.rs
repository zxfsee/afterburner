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

#[cfg(test)]
use burn::backend::wgpu::WgpuDevice;

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
) -> i32
where
    B::Device: train::runtime::ExplicitParticipantDevice,
{
    match args.task {
        TrainTask::Mnist => match train::train::<B>(mnist_config, device) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                2
            }
        },
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

#[derive(Debug, Clone, PartialEq)]
struct TrainArgs {
    task: TrainTask,
    batch_size: Option<usize>,
    num_workers: Option<usize>,
    num_epochs: Option<usize>,
    world_size: Option<usize>,
    device_group: Option<String>,
    participant_devices: Option<Vec<String>>,
    checkpoint_group: Option<String>,
    max_train_items: Option<usize>,
    max_valid_items: Option<usize>,
    dataset_manifest: Option<PathBuf>,
    tokenizer_profile: Option<PathBuf>,
    token_cache: Option<PathBuf>,
    resume_epoch: Option<usize>,
    max_validation_loss: Option<f64>,
    max_validation_perplexity: Option<f64>,
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
        max_validation_loss: args.max_validation_loss,
        max_validation_perplexity: args.max_validation_perplexity,
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
    if let Some(world_size) = args.world_size {
        config.world_size = world_size;
    }
    if let Some(device_group) = &args.device_group {
        config.device_group = device_group.clone();
    }
    if let Some(participant_devices) = &args.participant_devices {
        config.participant_devices = participant_devices.clone();
    }
    if let Some(checkpoint_group) = &args.checkpoint_group {
        config.checkpoint_group = checkpoint_group.clone();
    }
    config.resume_epoch = args.resume_epoch;
    if let Some(max_train_items) = args.max_train_items {
        config.max_train_items = max_train_items;
    }
    if let Some(max_valid_items) = args.max_valid_items {
        config.max_valid_items = max_valid_items;
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
        world_size: None,
        device_group: None,
        participant_devices: None,
        checkpoint_group: None,
        max_train_items: None,
        max_valid_items: None,
        dataset_manifest: None,
        tokenizer_profile: None,
        token_cache: None,
        resume_epoch: None,
        max_validation_loss: None,
        max_validation_perplexity: None,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--task" => parsed.task = parse_task(&mut args)?,
            "--batch-size" => parsed.batch_size = Some(parse_value(&mut args, "--batch-size")?),
            "--num-workers" => parsed.num_workers = Some(parse_value(&mut args, "--num-workers")?),
            "--num-epochs" => parsed.num_epochs = Some(parse_value(&mut args, "--num-epochs")?),
            "--world-size" => parsed.world_size = Some(parse_value(&mut args, "--world-size")?),
            "--device-group" => {
                parsed.device_group =
                    Some(parse_non_empty_string_value(&mut args, "--device-group")?)
            }
            "--participant-devices" => {
                parsed.participant_devices =
                    Some(parse_device_refs(&mut args, "--participant-devices")?)
            }
            "--checkpoint-group" => {
                parsed.checkpoint_group = Some(parse_non_empty_string_value(
                    &mut args,
                    "--checkpoint-group",
                )?)
            }
            "--max-train-items" => {
                parsed.max_train_items = Some(parse_value(&mut args, "--max-train-items")?)
            }
            "--max-valid-items" => {
                parsed.max_valid_items = Some(parse_value(&mut args, "--max-valid-items")?)
            }
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
            "--max-validation-loss" => {
                parsed.max_validation_loss = Some(parse_value(&mut args, "--max-validation-loss")?)
            }
            "--max-validation-perplexity" => {
                parsed.max_validation_perplexity =
                    Some(parse_value(&mut args, "--max-validation-perplexity")?)
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
            _ if arg.starts_with("--world-size=") => {
                parsed.world_size = Some(parse_inline_value(
                    arg.trim_start_matches("--world-size="),
                    "--world-size",
                )?)
            }
            _ if arg.starts_with("--device-group=") => {
                parsed.device_group = Some(parse_non_empty_inline_string(
                    arg.trim_start_matches("--device-group="),
                    "--device-group",
                )?)
            }
            _ if arg.starts_with("--participant-devices=") => {
                parsed.participant_devices = Some(parse_device_ref_list(
                    arg.trim_start_matches("--participant-devices="),
                    "--participant-devices",
                )?)
            }
            _ if arg.starts_with("--checkpoint-group=") => {
                parsed.checkpoint_group = Some(parse_non_empty_inline_string(
                    arg.trim_start_matches("--checkpoint-group="),
                    "--checkpoint-group",
                )?)
            }
            _ if arg.starts_with("--max-train-items=") => {
                parsed.max_train_items = Some(parse_inline_value(
                    arg.trim_start_matches("--max-train-items="),
                    "--max-train-items",
                )?)
            }
            _ if arg.starts_with("--max-valid-items=") => {
                parsed.max_valid_items = Some(parse_inline_value(
                    arg.trim_start_matches("--max-valid-items="),
                    "--max-valid-items",
                )?)
            }
            _ if arg.starts_with("--max-validation-loss=") => {
                parsed.max_validation_loss = Some(parse_inline_value(
                    arg.trim_start_matches("--max-validation-loss="),
                    "--max-validation-loss",
                )?)
            }
            _ if arg.starts_with("--max-validation-perplexity=") => {
                parsed.max_validation_perplexity = Some(parse_inline_value(
                    arg.trim_start_matches("--max-validation-perplexity="),
                    "--max-validation-perplexity",
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
    if matches!(parsed.world_size, Some(0) | Some(1)) {
        return Err(format!(
            "--world-size must be > 1 for the explicit DP runtime path\n{}",
            usage()
        ));
    }
    if matches!(parsed.max_train_items, Some(0)) {
        return Err(format!("--max-train-items must be > 0\n{}", usage()));
    }
    if matches!(parsed.max_valid_items, Some(0)) {
        return Err(format!("--max-valid-items must be > 0\n{}", usage()));
    }
    if matches!(parsed.resume_epoch, Some(0)) {
        return Err(format!("--resume-epoch must be > 0\n{}", usage()));
    }
    if parsed.task == TrainTask::Mnist
        && parsed.resume_epoch.is_some()
        && parsed.checkpoint_group.is_none()
    {
        return Err(format!(
            "--resume-epoch requires --checkpoint-group\n{}",
            usage()
        ));
    }
    if parsed.world_size.is_some() && parsed.device_group.is_none() {
        return Err(format!("--world-size requires --device-group\n{}", usage()));
    }
    if parsed.world_size.is_some() && parsed.participant_devices.is_none() {
        return Err(format!(
            "--world-size requires --participant-devices\n{}",
            usage()
        ));
    }
    if parsed.device_group.is_some() && parsed.world_size.is_none() {
        return Err(format!("--device-group requires --world-size\n{}", usage()));
    }
    if parsed.participant_devices.is_some() && parsed.world_size.is_none() {
        return Err(format!(
            "--participant-devices requires --world-size\n{}",
            usage()
        ));
    }
    if let (Some(world_size), Some(participant_devices)) =
        (parsed.world_size, parsed.participant_devices.as_ref())
        && participant_devices.len() != world_size
    {
        return Err(format!(
            "--participant-devices must contain exactly {world_size} entries for --world-size {world_size}\n{}",
            usage()
        ));
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
    if parsed.task == TrainTask::Text && parsed.world_size.is_some() {
        return Err(format!(
            "--task text does not support the explicit DP runtime path yet\n{}",
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

fn parse_non_empty_string_value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    let value = parse_string_value(args, flag)?;
    parse_non_empty_inline_string(value.as_str(), flag)
}

fn parse_non_empty_inline_string(value: &str, flag: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{flag} must be non-empty\n{}", usage()));
    }
    Ok(trimmed.to_string())
}

fn parse_device_refs<I>(args: &mut I, flag: &str) -> Result<Vec<String>, String>
where
    I: Iterator<Item = String>,
{
    let value = parse_string_value(args, flag)?;
    parse_device_ref_list(value.as_str(), flag)
}

fn parse_device_ref_list(value: &str, flag: &str) -> Result<Vec<String>, String> {
    if value.trim().is_empty() {
        return Err(format!(
            "{flag} must contain at least one device ref\n{}",
            usage()
        ));
    }

    let mut values = Vec::new();
    for entry in value.split(',') {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Err(format!(
                "{flag} entries must be non-empty comma-separated device refs\n{}",
                usage()
            ));
        }
        values.push(trimmed.to_string());
    }
    Ok(values)
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
    "usage: afterburner train [--task mnist|text] [--batch-size N] [--num-workers N] [--num-epochs N] [--world-size N --device-group NAME --participant-devices REF[,REF...]] [--checkpoint-group NAME] [--max-train-items N] [--max-valid-items N] [--dataset-manifest PATH --tokenizer-profile PATH --token-cache PATH [--resume-epoch N] [--max-validation-loss F64] [--max-validation-perplexity F64]]"
}

#[cfg(test)]
fn runtime_device_ref_to_wgpu(value: &str) -> Result<WgpuDevice, String> {
    match value {
        "default" => Ok(WgpuDevice::DefaultDevice),
        "cpu" => Ok(WgpuDevice::Cpu),
        _ if value.starts_with("discrete:") => value
            .trim_start_matches("discrete:")
            .parse::<usize>()
            .map(WgpuDevice::DiscreteGpu)
            .map_err(|_| format!("invalid discrete device ref `{value}`")),
        _ if value.starts_with("integrated:") => value
            .trim_start_matches("integrated:")
            .parse::<usize>()
            .map(WgpuDevice::IntegratedGpu)
            .map_err(|_| format!("invalid integrated device ref `{value}`")),
        _ if value.starts_with("virtual:") => value
            .trim_start_matches("virtual:")
            .parse::<usize>()
            .map(WgpuDevice::VirtualGpu)
            .map_err(|_| format!("invalid virtual device ref `{value}`")),
        _ if value.starts_with("existing:") => value
            .trim_start_matches("existing:")
            .parse::<u32>()
            .map(WgpuDevice::Existing)
            .map_err(|_| format!("invalid existing device ref `{value}`")),
        _ => Err(format!("unknown participant device ref `{value}`")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::WgpuDevice;
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
                world_size: None,
                device_group: None,
                participant_devices: None,
                checkpoint_group: None,
                max_train_items: None,
                max_valid_items: None,
                dataset_manifest: None,
                tokenizer_profile: None,
                token_cache: None,
                resume_epoch: None,
                max_validation_loss: None,
                max_validation_perplexity: None,
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
            world_size: None,
            device_group: None,
            participant_devices: None,
            checkpoint_group: None,
            max_train_items: None,
            max_valid_items: None,
            dataset_manifest: None,
            tokenizer_profile: None,
            token_cache: None,
            resume_epoch: None,
            max_validation_loss: None,
            max_validation_perplexity: None,
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
    fn parse_args_accepts_explicit_single_node_dp_runtime_flags() {
        let args = vec![
            "--world-size".to_string(),
            "2".to_string(),
            "--device-group".to_string(),
            "single-node:cpu".to_string(),
            "--participant-devices".to_string(),
            "cpu0,cpu1".to_string(),
            "--checkpoint-group".to_string(),
            "group-a".to_string(),
            "--max-train-items".to_string(),
            "64".to_string(),
            "--max-valid-items".to_string(),
            "32".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse dp args");
        assert_eq!(parsed.world_size, Some(2));
        assert_eq!(parsed.device_group.as_deref(), Some("single-node:cpu"));
        assert_eq!(
            parsed.participant_devices,
            Some(vec!["cpu0".to_string(), "cpu1".to_string()])
        );
        assert_eq!(parsed.checkpoint_group.as_deref(), Some("group-a"));
        assert_eq!(parsed.max_train_items, Some(64));
        assert_eq!(parsed.max_valid_items, Some(32));
    }

    #[test]
    fn parse_args_rejects_world_size_without_device_group() {
        let args = vec!["--world-size".to_string(), "2".to_string()];
        let err = parse_args(args.into_iter()).expect_err("world size without device group");
        assert!(err.contains("--world-size requires --device-group"));
    }

    #[test]
    fn parse_args_rejects_empty_device_group_inline() {
        let args = vec![
            "--world-size=2".to_string(),
            "--device-group=".to_string(),
            "--participant-devices=cpu0,cpu1".to_string(),
        ];
        let err = parse_args(args.into_iter()).expect_err("empty device group must fail");
        assert!(err.contains("--device-group must be non-empty"));
    }

    #[test]
    fn parse_args_rejects_world_size_without_participant_devices() {
        let args = vec![
            "--world-size".to_string(),
            "2".to_string(),
            "--device-group".to_string(),
            "single-node:cpu".to_string(),
        ];
        let err = parse_args(args.into_iter()).expect_err("world size without participant devices");
        assert!(err.contains("--world-size requires --participant-devices"));
    }

    #[test]
    fn parse_args_rejects_resume_without_checkpoint_group() {
        let args = vec!["--resume-epoch".to_string(), "1".to_string()];
        let err = parse_args(args.into_iter()).expect_err("resume without checkpoint group");
        assert!(err.contains("--resume-epoch requires --checkpoint-group"));
    }

    #[test]
    fn parse_args_rejects_empty_participant_device_entries() {
        let args = vec![
            "--world-size=2".to_string(),
            "--device-group=single-node:wgpu".to_string(),
            "--participant-devices=default, ".to_string(),
        ];
        let err = parse_args(args.into_iter()).expect_err("empty participant device entry");
        assert!(err.contains("--participant-devices entries must be non-empty"));
    }

    #[test]
    fn parse_args_rejects_text_dp_path_until_later_queue_item() {
        let args = vec![
            "--task".to_string(),
            "text".to_string(),
            "--dataset-manifest".to_string(),
            "fixtures/pretraining_dataset_manifest.example.json".to_string(),
            "--tokenizer-profile".to_string(),
            "fixtures/text_tokenizer_packing_profile.example.json".to_string(),
            "--token-cache".to_string(),
            "fixtures/text_token_cache.example.json".to_string(),
            "--world-size".to_string(),
            "2".to_string(),
            "--device-group".to_string(),
            "single-node:cpu".to_string(),
            "--participant-devices".to_string(),
            "cpu0,cpu1".to_string(),
        ];

        let err = parse_args(args.into_iter()).expect_err("text dp must fail");
        assert!(err.contains("does not support the explicit DP runtime path yet"));
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
            "--max-validation-loss".to_string(),
            "4.0".to_string(),
            "--max-validation-perplexity=80.0".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse text args");
        assert_eq!(parsed.task, TrainTask::Text);
        assert_eq!(parsed.resume_epoch, Some(1));
        assert_eq!(parsed.max_validation_loss, Some(4.0));
        assert_eq!(parsed.max_validation_perplexity, Some(80.0));
        assert_eq!(
            parsed.dataset_manifest.as_deref(),
            Some(Path::new(
                "fixtures/pretraining_dataset_manifest.example.json"
            ))
        );
    }

    #[test]
    fn wgpu_device_refs_parse_as_distinct_inventory_entries() {
        let parsed = [
            "default".to_string(),
            "cpu".to_string(),
            "integrated:0".to_string(),
            "discrete:0".to_string(),
        ]
        .into_iter()
        .map(|value| super::runtime_device_ref_to_wgpu(value.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .expect("device refs parse");

        assert_eq!(
            parsed,
            vec![
                WgpuDevice::DefaultDevice,
                WgpuDevice::Cpu,
                WgpuDevice::IntegratedGpu(0),
                WgpuDevice::DiscreteGpu(0),
            ]
        );
    }
}
