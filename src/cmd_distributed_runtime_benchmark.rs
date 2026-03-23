use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use afterburner::observability::emit_event;
use serde_json::json;

const DEFAULT_OUT_PATH: &str = "artifacts/train/distributed_runtime_benchmark_run.json";

#[derive(Debug)]
enum DistributedRuntimeBenchmarkError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
    BenchmarkFailed(String),
}

impl std::fmt::Display for DistributedRuntimeBenchmarkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
            Self::BenchmarkFailed(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedRuntimeBenchmarkError {}

impl From<std::io::Error> for DistributedRuntimeBenchmarkError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Args {
    artifact_version: String,
    model_size: String,
    node_count: u64,
    gpus_per_node: u64,
    world_size: u64,
    dp: u64,
    tp: u64,
    pp: u64,
    sp_cp: u64,
    ep: u64,
    gas: u64,
    microbatch_size: u64,
    zero_stage: u64,
    backend: String,
    system: String,
    gpu_model: String,
    burn_version: String,
    seed: u64,
    measurement_window_steps: u64,
    benchmark_command: String,
    step_time_ms: f64,
    tokens_per_sec: f64,
    mfu: f64,
    max_memory_bytes: u64,
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

fn run_inner<I>(args: I) -> Result<PathBuf, DistributedRuntimeBenchmarkError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let started = Instant::now();
    let output = Command::new("sh")
        .arg("-lc")
        .arg(&args.benchmark_command)
        .output()?;
    let benchmark_duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    if !output.status.success() {
        return Err(DistributedRuntimeBenchmarkError::BenchmarkFailed(format!(
            "benchmark command failed with status {}: {}",
            output
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".to_string()),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let artifact = json!({
        "schema_version": "1",
        "artifact_version": args.artifact_version,
        "model_size": args.model_size,
        "node_count": args.node_count,
        "gpus_per_node": args.gpus_per_node,
        "world_size": args.world_size,
        "parallelism": {
            "dp": args.dp,
            "tp": args.tp,
            "pp": args.pp,
            "sp_cp": args.sp_cp,
            "ep": args.ep,
            "gas": args.gas,
            "microbatch_size": args.microbatch_size,
            "zero_stage": args.zero_stage,
        },
        "benchmark_config": {
            "seed": args.seed,
            "measurement_window_steps": args.measurement_window_steps,
            "benchmark_command": args.benchmark_command,
        },
        "metrics": {
            "mfu": args.mfu,
            "step_time_ms": args.step_time_ms,
            "tokens_per_sec": args.tokens_per_sec,
            "max_memory_bytes": args.max_memory_bytes,
        },
        "runtime_settings": {
            "backend": args.backend,
            "weights_dtype": "f32",
            "activation_dtype": "f32",
            "quantization": "none",
        },
        "environment_fingerprint": {
            "system": args.system,
            "gpu_model": args.gpu_model,
            "burn_version": args.burn_version,
        },
        "benchmark_duration_ms": benchmark_duration_ms,
        "benchmark_status": "passed",
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&artifact).map_err(|err| {
        DistributedRuntimeBenchmarkError::Parse(format!("serialize benchmark artifact: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "profile_cli",
        "distributed_runtime_benchmark_run_written",
        json!({
            "artifact_path": args.out_path.display().to_string(),
            "artifact": artifact,
        }),
    );

    Ok(args.out_path)
}

fn parse_args<I>(args: I) -> Result<Args, DistributedRuntimeBenchmarkError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut parsed = Args {
        artifact_version: String::new(),
        model_size: String::new(),
        node_count: 0,
        gpus_per_node: 0,
        world_size: 0,
        dp: 0,
        tp: 0,
        pp: 0,
        sp_cp: 0,
        ep: 0,
        gas: 0,
        microbatch_size: 0,
        zero_stage: 0,
        backend: String::new(),
        system: String::new(),
        gpu_model: String::new(),
        burn_version: String::new(),
        seed: 0,
        measurement_window_steps: 0,
        benchmark_command: String::new(),
        step_time_ms: 0.0,
        tokens_per_sec: 0.0,
        mfu: 0.0,
        max_memory_bytes: 0,
        out_path: PathBuf::from(DEFAULT_OUT_PATH),
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact-version" => {
                parsed.artifact_version = parse_value(&mut args, "--artifact-version")?
            }
            "--model-size" => parsed.model_size = parse_value(&mut args, "--model-size")?,
            "--node-count" => parsed.node_count = parse_value(&mut args, "--node-count")?,
            "--gpus-per-node" => parsed.gpus_per_node = parse_value(&mut args, "--gpus-per-node")?,
            "--world-size" => parsed.world_size = parse_value(&mut args, "--world-size")?,
            "--dp" => parsed.dp = parse_value(&mut args, "--dp")?,
            "--tp" => parsed.tp = parse_value(&mut args, "--tp")?,
            "--pp" => parsed.pp = parse_value(&mut args, "--pp")?,
            "--sp-cp" => parsed.sp_cp = parse_value(&mut args, "--sp-cp")?,
            "--ep" => parsed.ep = parse_value(&mut args, "--ep")?,
            "--gas" => parsed.gas = parse_value(&mut args, "--gas")?,
            "--microbatch-size" => {
                parsed.microbatch_size = parse_value(&mut args, "--microbatch-size")?
            }
            "--zero-stage" => parsed.zero_stage = parse_value(&mut args, "--zero-stage")?,
            "--backend" => parsed.backend = parse_value(&mut args, "--backend")?,
            "--system" => parsed.system = parse_value(&mut args, "--system")?,
            "--gpu-model" => parsed.gpu_model = parse_value(&mut args, "--gpu-model")?,
            "--burn-version" => parsed.burn_version = parse_value(&mut args, "--burn-version")?,
            "--seed" => parsed.seed = parse_value(&mut args, "--seed")?,
            "--measurement-window-steps" => {
                parsed.measurement_window_steps =
                    parse_value(&mut args, "--measurement-window-steps")?
            }
            "--benchmark-command" => {
                parsed.benchmark_command = parse_string_value(&mut args, "--benchmark-command")?
            }
            "--step-time-ms" => parsed.step_time_ms = parse_value(&mut args, "--step-time-ms")?,
            "--tokens-per-sec" => {
                parsed.tokens_per_sec = parse_value(&mut args, "--tokens-per-sec")?
            }
            "--mfu" => parsed.mfu = parse_value(&mut args, "--mfu")?,
            "--max-memory-bytes" => {
                parsed.max_memory_bytes = parse_value(&mut args, "--max-memory-bytes")?
            }
            "--out" => parsed.out_path = PathBuf::from(parse_string_value(&mut args, "--out")?),
            _ => {
                return Err(DistributedRuntimeBenchmarkError::InvalidArg(format!(
                    "unknown argument for profile distributed-runtime-benchmark: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    if parsed.artifact_version.is_empty()
        || parsed.model_size.is_empty()
        || parsed.backend.is_empty()
        || parsed.system.is_empty()
        || parsed.gpu_model.is_empty()
        || parsed.burn_version.is_empty()
        || parsed.benchmark_command.trim().is_empty()
    {
        return Err(DistributedRuntimeBenchmarkError::InvalidArg(format!(
            "missing required benchmark fields\n{}",
            usage()
        )));
    }

    Ok(parsed)
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DistributedRuntimeBenchmarkError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedRuntimeBenchmarkError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedRuntimeBenchmarkError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn parse_string_value<I>(
    args: &mut I,
    flag: &str,
) -> Result<String, DistributedRuntimeBenchmarkError>
where
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedRuntimeBenchmarkError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    if value.trim().is_empty() {
        return Err(DistributedRuntimeBenchmarkError::InvalidArg(format!(
            "{flag} must be non-empty\n{}",
            usage()
        )));
    }
    Ok(value)
}

fn usage() -> &'static str {
    "usage: afterburner profile distributed-runtime-benchmark --artifact-version V --model-size SIZE --node-count N --gpus-per-node N --world-size N --dp N --tp N --pp N --sp-cp N --ep N --gas N --microbatch-size N --zero-stage N --backend NAME --system NAME --gpu-model NAME --burn-version V --seed N --measurement-window-steps N --benchmark-command CMD --step-time-ms F64 --tokens-per-sec F64 --mfu F64 --max-memory-bytes N [--out PATH]"
}
