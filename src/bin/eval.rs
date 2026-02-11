use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use afterburner::data::test_loader;
use afterburner::infer::{default_weights_path, load_model};
use afterburner::observability::json_escape;
use burn::backend::ndarray::NdArray;
use burn::prelude::*;

type CpuBackend = NdArray<f32>;

#[derive(Debug)]
enum EvalError {
    InvalidArg(String),
    Io(std::io::Error),
    Infer(afterburner::infer::InferError),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Infer(err) => write!(f, "inference error: {err:?}"),
        }
    }
}

fn usage() -> &'static str {
    "usage: eval [artifact_path] [--seed N] [--batch-size N] [--max-batches N] [--out PATH]"
}

impl std::error::Error for EvalError {}

impl EvalError {
    fn unknown_arg(arg: &str) -> Self {
        Self::InvalidArg(format!("unknown argument: {arg}\n{}", usage()))
    }

    fn invalid_arg(msg: String) -> Self {
        Self::InvalidArg(format!("{msg}\n{}", usage()))
    }
}

impl From<std::io::Error> for EvalError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<afterburner::infer::InferError> for EvalError {
    fn from(value: afterburner::infer::InferError) -> Self {
        Self::Infer(value)
    }
}

#[derive(Debug)]
struct EvalArgs {
    artifact: PathBuf,
    seed: u64,
    batch_size: usize,
    max_batches: usize,
    out_path: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!(
            "{{\"event\":\"eval_error\",\"kind\":\"cli\",\"detail\":\"{}\"}}",
            json_escape(&err.to_string())
        );
        std::process::exit(2);
    }
}

fn run() -> Result<(), EvalError> {
    let args = parse_args(std::env::args().skip(1))?;
    let device = <CpuBackend as Backend>::Device::default();
    <CpuBackend as Backend>::seed(&device, args.seed);

    let model = load_model::<CpuBackend>(&args.artifact, &device)?;
    let loader = test_loader::<CpuBackend>(args.batch_size, 0, device);

    let start = Instant::now();
    let mut total = 0usize;
    let mut correct = 0usize;
    let mut seen_batches = 0usize;

    for batch in loader.iter() {
        if seen_batches >= args.max_batches {
            break;
        }

        let logits = model.forward(batch.images);
        let predicted = logits.argmax(1);

        let pred = predicted.to_data();
        let truth = batch.targets.to_data();
        let pred_vec: Vec<i64> = pred.iter().collect();
        let truth_vec: Vec<i64> = truth.iter().collect();

        let matched = pred_vec
            .iter()
            .zip(truth_vec.iter())
            .filter(|(p, t)| p == t)
            .count();
        correct += matched;
        total += truth_vec.len();
        seen_batches += 1;
    }

    if total == 0 {
        return Err(EvalError::invalid_arg("no samples evaluated".to_string()));
    }

    let accuracy = correct as f64 / total as f64;
    let elapsed_ms = start.elapsed().as_millis();
    let summary = format!(
        "{{\n  \"event\": \"mnist_eval_summary\",\n  \"artifact\": \"{}\",\n  \"seed\": {},\n  \"batch_size\": {},\n  \"max_batches\": {},\n  \"batches_evaluated\": {},\n  \"samples\": {},\n  \"correct\": {},\n  \"accuracy\": {:.8},\n  \"elapsed_ms\": {}\n}}",
        json_escape(&args.artifact.display().to_string()),
        args.seed,
        args.batch_size,
        args.max_batches,
        seen_batches,
        total,
        correct,
        accuracy,
        elapsed_ms
    );

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out_path, summary)?;

    eprintln!(
        "{{\"event\":\"mnist_eval_written\",\"out\":\"{}\",\"accuracy\":{:.8}}}",
        json_escape(&args.out_path.display().to_string()),
        accuracy
    );

    Ok(())
}

fn parse_args<I>(args: I) -> Result<EvalArgs, EvalError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let artifact = if matches!(args.peek(), Some(v) if !v.starts_with("--")) {
        match args.next() {
            Some(path) => PathBuf::from(path),
            None => default_weights_path(),
        }
    } else {
        default_weights_path()
    };

    let mut seed = 42u64;
    let mut batch_size = 128usize;
    let mut max_batches = 8usize;
    let mut out_path = PathBuf::from("artifacts/eval/mnist_eval_summary.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => seed = parse_value(&mut args, "--seed")?,
            "--batch-size" => batch_size = parse_value(&mut args, "--batch-size")?,
            "--max-batches" => max_batches = parse_value(&mut args, "--max-batches")?,
            "--out" => {
                let value = args
                    .next()
                    .ok_or_else(|| EvalError::invalid_arg("missing value for --out".to_string()))?;
                out_path = PathBuf::from(value);
            }
            _ => return Err(EvalError::unknown_arg(&arg)),
        }
    }

    if batch_size == 0 {
        return Err(EvalError::invalid_arg(
            "--batch-size must be > 0".to_string(),
        ));
    }
    if max_batches == 0 {
        return Err(EvalError::invalid_arg(
            "--max-batches must be > 0".to_string(),
        ));
    }

    Ok(EvalArgs {
        artifact,
        seed,
        batch_size,
        max_batches,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, EvalError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args
        .next()
        .ok_or_else(|| EvalError::invalid_arg(format!("missing value for {flag}")))?;
    value
        .parse::<T>()
        .map_err(|_| EvalError::invalid_arg(format!("invalid value for {flag}: {value}")))
}

#[cfg(test)]
mod tests {
    use super::{default_weights_path, parse_args};
    use std::path::PathBuf;

    #[test]
    fn parse_args_defaults_artifact_when_not_provided() {
        let args = vec![
            "--seed".to_string(),
            "42".to_string(),
            "--batch-size".to_string(),
            "64".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse args");
        assert_eq!(parsed.artifact, default_weights_path());
        assert_eq!(parsed.seed, 42);
        assert_eq!(parsed.batch_size, 64);
    }

    #[test]
    fn parse_args_accepts_explicit_artifact_path() {
        let args = vec![
            "artifacts/inference/0.1.0/model.mpk".to_string(),
            "--max-batches".to_string(),
            "4".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse args");
        assert_eq!(
            parsed.artifact,
            PathBuf::from("artifacts/inference/0.1.0/model.mpk")
        );
        assert_eq!(parsed.max_batches, 4);
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let args = vec!["--bogus".to_string()];
        let err = parse_args(args.into_iter()).expect_err("unknown flag should fail");
        assert!(
            err.to_string().contains("unknown argument: --bogus"),
            "unexpected error: {err}"
        );
    }
}
