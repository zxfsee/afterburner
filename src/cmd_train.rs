use std::any::Any;

use afterburner::model::ModelConfig;
use afterburner::observability::emit_event;
use afterburner::train;
use burn::prelude::*;
use burn::{
    backend::ndarray::NdArray,
    backend::wgpu::{Metal, Wgpu},
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

    let config = training_config_from_env(&parsed);

    if backend == "cpu" {
        let device = <CpuBackend as Backend>::Device::default();
        train::train::<CpuAutodiff>(config, device);
        return 0;
    }

    let gpu_result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match backend.as_str() {
            "metal" => {
                let device = <MetalBackend as Backend>::Device::default();
                train::train::<MetalAutodiff>(config, device);
            }
            _ => {
                let device = <GpuBackend as Backend>::Device::default();
                train::train::<GpuAutodiff>(config, device);
            }
        }));

    if let Err(payload) = gpu_result {
        if is_missing_gpu_adapter_panic(payload.as_ref()) {
            emit_event(
                "warn",
                "train_cli",
                "backend_fallback",
                json!({"from":backend,"to":"cpu","reason":"no_adapter"}),
            );
            let cpu_config = training_config_from_env(&parsed);
            let device = <CpuBackend as Backend>::Device::default();
            train::train::<CpuAutodiff>(cpu_config, device);
            return 0;
        }

        std::panic::resume_unwind(payload);
    }

    0
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrainArgs {
    batch_size: Option<usize>,
    num_workers: Option<usize>,
    num_epochs: Option<usize>,
}

fn training_config_from_env(args: &TrainArgs) -> train::TrainingConfig {
    // Explicit runtime knob to mirror production deployment variance:
    // same code + same artifact contract, different execution target.
    // Single override point so CI / experiments can redirect outputs without code changes.
    // The artifact contract lives under `<ARTIFACTS_DIR>/inference/`.
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
        batch_size: None,
        num_workers: None,
        num_epochs: None,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--batch-size" => parsed.batch_size = Some(parse_value(&mut args, "--batch-size")?),
            "--num-workers" => parsed.num_workers = Some(parse_value(&mut args, "--num-workers")?),
            "--num-epochs" => parsed.num_epochs = Some(parse_value(&mut args, "--num-epochs")?),
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
    "usage: afterburner train [--batch-size N] [--num-workers N] [--num-epochs N]"
}

#[cfg(test)]
mod tests {
    use super::{TrainArgs, parse_args, training_config_from_env};

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
                batch_size: Some(32),
                num_workers: Some(4),
                num_epochs: Some(1),
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
            batch_size: None,
            num_workers: None,
            num_epochs: Some(1),
        });
        assert_eq!(config.num_epochs, 1);
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let args = vec!["--bogus".to_string()];
        let err = parse_args(args.into_iter()).expect_err("unknown flag must fail");
        assert!(err.contains("unknown argument for train: --bogus"));
    }
}
