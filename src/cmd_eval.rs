use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use afterburner::data::test_loader;
use afterburner::infer::{default_weights_path, load_model};
use afterburner::observability::{emit_event, json_escape};
use burn::backend::ndarray::NdArray;
use burn::prelude::*;
use serde_json::json;

type CpuBackend = NdArray<f32>;

#[derive(Debug)]
enum EvalError {
    InvalidArg(String),
    Io(std::io::Error),
    Infer(afterburner::infer::InferError),
    AccuracyBelowBaseline { accuracy: f64, min_accuracy: f64 },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Infer(err) => write!(f, "inference error: {err:?}"),
            Self::AccuracyBelowBaseline {
                accuracy,
                min_accuracy,
            } => write!(
                f,
                "eval accuracy below baseline: accuracy={accuracy:.8} min_accuracy={min_accuracy:.8}"
            ),
        }
    }
}

fn usage() -> &'static str {
    "usage: afterburner eval [artifact_path] [--artifact PATH] [--seed N] [--batch-size N] [--max-batches N] [--min-accuracy F64] [--out PATH]"
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
    min_accuracy: Option<f64>,
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

    if let Err(err) = run_inner(args.into_iter()) {
        emit_event(
            "error",
            "eval_cli",
            "eval_error",
            json!({"kind": "cli", "detail": err.to_string()}),
        );
        return 2;
    }

    0
}

fn run_inner<I>(args: I) -> Result<(), EvalError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
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

    emit_event(
        "info",
        "eval_cli",
        "mnist_eval_written",
        json!({
            "out": args.out_path.display().to_string(),
            "accuracy": accuracy
        }),
    );

    if let Some(min_accuracy) = args.min_accuracy {
        if let Err(err) = enforce_accuracy_gate(accuracy, min_accuracy) {
            emit_event(
                "error",
                "eval_cli",
                "mnist_eval_gate_failed",
                json!({"accuracy": accuracy, "min_accuracy": min_accuracy}),
            );
            return Err(err);
        }
        emit_event(
            "info",
            "eval_cli",
            "mnist_eval_gate_passed",
            json!({"accuracy": accuracy, "min_accuracy": min_accuracy}),
        );
    }

    Ok(())
}

fn parse_args<I>(args: I) -> Result<EvalArgs, EvalError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut artifact = if matches!(args.peek(), Some(v) if !v.starts_with("--")) {
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
    let mut min_accuracy = None::<f64>;
    let mut out_path = PathBuf::from("artifacts/eval/mnist_eval_summary.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => {
                let value = args.next().ok_or_else(|| {
                    EvalError::invalid_arg("missing value for --artifact".to_string())
                })?;
                artifact = PathBuf::from(value);
            }
            "--seed" => seed = parse_value(&mut args, "--seed")?,
            "--batch-size" => batch_size = parse_value(&mut args, "--batch-size")?,
            "--max-batches" => max_batches = parse_value(&mut args, "--max-batches")?,
            "--min-accuracy" => min_accuracy = Some(parse_value(&mut args, "--min-accuracy")?),
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
    if let Some(min_accuracy) = min_accuracy
        && !(0.0..=1.0).contains(&min_accuracy)
    {
        return Err(EvalError::invalid_arg(
            "--min-accuracy must be in range [0.0, 1.0]".to_string(),
        ));
    }

    Ok(EvalArgs {
        artifact,
        seed,
        batch_size,
        max_batches,
        min_accuracy,
        out_path,
    })
}

fn enforce_accuracy_gate(accuracy: f64, min_accuracy: f64) -> Result<(), EvalError> {
    if accuracy < min_accuracy {
        return Err(EvalError::AccuracyBelowBaseline {
            accuracy,
            min_accuracy,
        });
    }
    Ok(())
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
    use super::{default_weights_path, enforce_accuracy_gate, parse_args};
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
    fn parse_args_accepts_named_artifact_flag() {
        let args = vec![
            "--artifact".to_string(),
            "artifacts/inference/0.2.0/model.mpk".to_string(),
            "--max-batches".to_string(),
            "4".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse args");
        assert_eq!(
            parsed.artifact,
            PathBuf::from("artifacts/inference/0.2.0/model.mpk")
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

    #[test]
    fn parse_args_accepts_min_accuracy() {
        let args = vec![
            "--min-accuracy".to_string(),
            "0.98".to_string(),
            "--max-batches".to_string(),
            "4".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse args");
        assert_eq!(parsed.min_accuracy, Some(0.98));
        assert_eq!(parsed.max_batches, 4);
    }

    #[test]
    fn parse_args_rejects_out_of_range_min_accuracy() {
        let args = vec!["--min-accuracy".to_string(), "1.5".to_string()];
        let err = parse_args(args.into_iter()).expect_err("must reject invalid min accuracy");
        assert!(err.to_string().contains("--min-accuracy must be in range"));
    }

    #[test]
    fn accuracy_gate_fails_below_threshold() {
        let err = enforce_accuracy_gate(0.9, 0.91).expect_err("must fail threshold");
        assert!(err.to_string().contains("below baseline"));
    }

    #[test]
    fn accuracy_gate_accepts_equal_or_above_threshold() {
        enforce_accuracy_gate(0.91, 0.91).expect("equal threshold should pass");
        enforce_accuracy_gate(0.92, 0.91).expect("higher accuracy should pass");
    }
}
