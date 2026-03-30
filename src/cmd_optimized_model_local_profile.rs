use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, path_payload_fields_with_extra, write_json_value,
};
use afterburner::observability::emit_event;
use serde_json::{Map, Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/eval/optimized_model_local_profile.json";

#[derive(Debug)]
enum OptimizedModelLocalProfileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for OptimizedModelLocalProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for OptimizedModelLocalProfileError {}

impl From<std::io::Error> for OptimizedModelLocalProfileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for OptimizedModelLocalProfileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize optimized model local profile: {err}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Args {
    package_contract_path: PathBuf,
    quality_metric: String,
    quality_value: f64,
    minimum_quality_value: f64,
    observed_latency_ms_p99: f64,
    observed_max_memory_bytes: u64,
    observed_package_bytes: u64,
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

fn run_inner<I>(args: I) -> Result<PathBuf, OptimizedModelLocalProfileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let package_contract = read_json_object(
        &args.package_contract_path,
        "optimized model package contract",
    )?;
    let packaging_inputs = required_object(
        &package_contract,
        "packaging_inputs",
        "optimized model package contract",
    )?;
    let optimization_profile_path = PathBuf::from(required_string(
        packaging_inputs,
        "optimization_profile_path",
        "optimized model package contract",
    )?);
    let optimization_profile =
        read_json_object(&optimization_profile_path, "model optimization profile")?;

    let package_target = required_string(
        &package_contract,
        "target_environment",
        "optimized model package contract",
    )?;
    let profile_target = required_string(
        &optimization_profile,
        "target_environment",
        "model optimization profile",
    )?;
    if package_target != profile_target {
        return Err(OptimizedModelLocalProfileError::Parse(format!(
            "package contract target_environment `{package_target}` does not match optimization profile target_environment `{profile_target}`"
        )));
    }

    let output_constraints = required_object(
        &optimization_profile,
        "output_constraints",
        "model optimization profile",
    )?;
    let latency_budget_ms_p99 = required_f64(
        output_constraints,
        "max_latency_ms_p99",
        "model optimization profile output_constraints",
    )?;
    let memory_budget_bytes = required_u64(
        output_constraints,
        "max_memory_bytes",
        "model optimization profile output_constraints",
    )?;
    let package_budget_bytes = required_u64(
        output_constraints,
        "max_package_bytes",
        "model optimization profile output_constraints",
    )?;

    let quality_passed = args.quality_value >= args.minimum_quality_value;
    let latency_passed = args.observed_latency_ms_p99 <= latency_budget_ms_p99;
    let memory_passed = args.observed_max_memory_bytes <= memory_budget_bytes;
    let package_size_passed = args.observed_package_bytes <= package_budget_bytes;
    let passed = quality_passed && latency_passed && memory_passed && package_size_passed;

    let profile = json!({
        "schema_version": "1",
        "package_contract_path": args.package_contract_path.display().to_string(),
        "optimization_profile_path": optimization_profile_path.display().to_string(),
        "input_artifact_version": required_string(&package_contract, "input_artifact_version", "optimized model package contract")?,
        "target_environment": package_target,
        "optimization_steps": required_array(&package_contract, "optimization_steps", "optimized model package contract")?,
        "runtime_precision_contract": required_object_value(&package_contract, "runtime_precision_contract", "optimized model package contract")?,
        "optimized_artifact_path": required_string(&package_contract, "optimized_artifact_path", "optimized model package contract")?,
        "export_format": required_string(&package_contract, "export_format", "optimized model package contract")?,
        "quality": {
            "metric": args.quality_metric,
            "observed": args.quality_value,
            "minimum_accepted": args.minimum_quality_value,
            "passed": quality_passed
        },
        "latency": {
            "observed_ms_p99": args.observed_latency_ms_p99,
            "budget_ms_p99": latency_budget_ms_p99,
            "passed": latency_passed
        },
        "memory": {
            "observed_bytes": args.observed_max_memory_bytes,
            "budget_bytes": memory_budget_bytes,
            "passed": memory_passed
        },
        "package_size": {
            "observed_bytes": args.observed_package_bytes,
            "budget_bytes": package_budget_bytes,
            "passed": package_size_passed
        },
        "passed": passed
    });

    write_json_value(&args.out_path, &profile)?;

    let mut extra = Map::new();
    extra.insert(
        "package_contract_path".to_string(),
        Value::from(args.package_contract_path.display().to_string()),
    );
    extra.insert(
        "optimization_profile_path".to_string(),
        Value::from(optimization_profile_path.display().to_string()),
    );
    emit_event(
        "info",
        "profile_cli",
        "optimized_model_local_profile_written",
        path_payload_fields_with_extra("profile_path", &args.out_path, "profile", profile, extra),
    );

    Ok(args.out_path)
}

fn read_json_object(
    path: &PathBuf,
    kind: &str,
) -> Result<Map<String, Value>, OptimizedModelLocalProfileError> {
    let text = std::fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        OptimizedModelLocalProfileError::Parse(format!(
            "parse {kind} json from {}: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        OptimizedModelLocalProfileError::Parse(format!(
            "{kind} at {} must be a JSON object",
            path.display()
        ))
    })
}

fn required_value<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<&'a Value, OptimizedModelLocalProfileError> {
    object.get(key).ok_or_else(|| {
        OptimizedModelLocalProfileError::Parse(format!("{kind} is missing required field `{key}`"))
    })
}

fn required_string(
    object: &Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<String, OptimizedModelLocalProfileError> {
    required_value(object, key, kind)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            OptimizedModelLocalProfileError::Parse(format!("{kind} field `{key}` must be a string"))
        })
}

fn required_f64(
    object: &Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<f64, OptimizedModelLocalProfileError> {
    required_value(object, key, kind)?.as_f64().ok_or_else(|| {
        OptimizedModelLocalProfileError::Parse(format!("{kind} field `{key}` must be a number"))
    })
}

fn required_u64(
    object: &Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<u64, OptimizedModelLocalProfileError> {
    required_value(object, key, kind)?.as_u64().ok_or_else(|| {
        OptimizedModelLocalProfileError::Parse(format!("{kind} field `{key}` must be a u64"))
    })
}

fn required_array(
    object: &Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<Value, OptimizedModelLocalProfileError> {
    let value = required_value(object, key, kind)?;
    if !value.is_array() {
        return Err(OptimizedModelLocalProfileError::Parse(format!(
            "{kind} field `{key}` must be an array"
        )));
    }
    Ok(value.clone())
}

fn required_object<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<&'a Map<String, Value>, OptimizedModelLocalProfileError> {
    required_value(object, key, kind)?
        .as_object()
        .ok_or_else(|| {
            OptimizedModelLocalProfileError::Parse(format!(
                "{kind} field `{key}` must be an object"
            ))
        })
}

fn required_object_value(
    object: &Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<Value, OptimizedModelLocalProfileError> {
    let value = required_value(object, key, kind)?;
    if !value.is_object() {
        return Err(OptimizedModelLocalProfileError::Parse(format!(
            "{kind} field `{key}` must be an object"
        )));
    }
    Ok(value.clone())
}

fn parse_args<I>(args: I) -> Result<Args, OptimizedModelLocalProfileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut parsed = Args {
        package_contract_path: PathBuf::new(),
        quality_metric: String::new(),
        quality_value: 0.0,
        minimum_quality_value: 0.0,
        observed_latency_ms_p99: 0.0,
        observed_max_memory_bytes: 0,
        observed_package_bytes: 0,
        out_path: PathBuf::from(DEFAULT_OUT_PATH),
    };
    let mut seen_package_contract = false;
    let mut seen_quality_metric = false;
    let mut seen_quality_value = false;
    let mut seen_minimum_quality_value = false;
    let mut seen_latency = false;
    let mut seen_memory = false;
    let mut seen_package_bytes = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--package-contract" => {
                parsed.package_contract_path =
                    PathBuf::from(parse_string_value(&mut args, "--package-contract")?);
                seen_package_contract = true;
            }
            "--quality-metric" => {
                parsed.quality_metric = parse_string_value(&mut args, "--quality-metric")?;
                seen_quality_metric = true;
            }
            "--quality-value" => {
                parsed.quality_value = parse_value(&mut args, "--quality-value")?;
                seen_quality_value = true;
            }
            "--minimum-quality-value" => {
                parsed.minimum_quality_value = parse_value(&mut args, "--minimum-quality-value")?;
                seen_minimum_quality_value = true;
            }
            "--observed-latency-ms-p99" => {
                parsed.observed_latency_ms_p99 =
                    parse_value(&mut args, "--observed-latency-ms-p99")?;
                seen_latency = true;
            }
            "--observed-max-memory-bytes" => {
                parsed.observed_max_memory_bytes =
                    parse_value(&mut args, "--observed-max-memory-bytes")?;
                seen_memory = true;
            }
            "--observed-package-bytes" => {
                parsed.observed_package_bytes = parse_value(&mut args, "--observed-package-bytes")?;
                seen_package_bytes = true;
            }
            "--out" => parsed.out_path = PathBuf::from(parse_string_value(&mut args, "--out")?),
            _ => {
                return Err(OptimizedModelLocalProfileError::InvalidArg(format!(
                    "unknown argument for profile optimized-model-local-profile: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    if !seen_package_contract {
        return Err(missing_arg("--package-contract"));
    }
    if !seen_quality_metric {
        return Err(missing_arg("--quality-metric"));
    }
    if !seen_quality_value {
        return Err(missing_arg("--quality-value"));
    }
    if !seen_minimum_quality_value {
        return Err(missing_arg("--minimum-quality-value"));
    }
    if !seen_latency {
        return Err(missing_arg("--observed-latency-ms-p99"));
    }
    if !seen_memory {
        return Err(missing_arg("--observed-max-memory-bytes"));
    }
    if !seen_package_bytes {
        return Err(missing_arg("--observed-package-bytes"));
    }

    Ok(parsed)
}

fn parse_string_value<I>(
    args: &mut std::iter::Peekable<I>,
    flag: &str,
) -> Result<String, OptimizedModelLocalProfileError>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| missing_arg(flag))
}

fn parse_value<T, I>(
    args: &mut std::iter::Peekable<I>,
    flag: &str,
) -> Result<T, OptimizedModelLocalProfileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = parse_string_value(args, flag)?;
    value.parse::<T>().map_err(|_| invalid_arg(flag))
}

fn missing_arg(flag: &str) -> OptimizedModelLocalProfileError {
    OptimizedModelLocalProfileError::InvalidArg(format!(
        "missing required argument {flag}\n{}",
        usage()
    ))
}

fn invalid_arg(flag: &str) -> OptimizedModelLocalProfileError {
    OptimizedModelLocalProfileError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
}

fn usage() -> &'static str {
    "usage: afterburner profile optimized-model-local-profile --package-contract PATH --quality-metric NAME --quality-value F64 --minimum-quality-value F64 --observed-latency-ms-p99 F64 --observed-max-memory-bytes N --observed-package-bytes N [--out PATH]"
}
