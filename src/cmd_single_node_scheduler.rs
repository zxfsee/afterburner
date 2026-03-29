use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const DEFAULT_LEASE_PATH: &str = "artifacts/deploy/gpu_scheduler_lease.json";

#[derive(Debug)]
enum SingleNodeSchedulerError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SingleNodeSchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SingleNodeSchedulerError {}

impl From<std::io::Error> for SingleNodeSchedulerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    job_path: PathBuf,
    inventory_path: PathBuf,
    out_lease: PathBuf,
    out_unit: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), SingleNodeSchedulerError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let job = load_job(&args.job_path)?;
    let inventory = load_inventory(&args.inventory_path)?;

    if job.gpu_count != 1 {
        return Err(SingleNodeSchedulerError::Parse(
            "single-node scheduler currently supports gpu_count = 1 only".to_string(),
        ));
    }

    let free_gpu = inventory
        .gpus
        .iter()
        .find(|gpu| gpu.state == "free")
        .ok_or_else(|| SingleNodeSchedulerError::Parse("no free gpu available".to_string()))?;

    let lease_id = format!("lease-{}", job.job_id);
    let service_unit = format!("afterburner-{}.service", job.job_id);

    let lease = json!({
        "schema_version": "1",
        "job_id": job.job_id,
        "lease_id": lease_id,
        "node_id": inventory.node_id,
        "gpu_ids": [free_gpu.gpu_id],
        "priority": job.priority,
        "checkpoint_mode": job.checkpoint_mode,
        "service_unit": service_unit,
    });

    let unit = format!(
        "[Unit]\nDescription=Afterburner job {job_id}\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory={working_directory}\nEnvironment=AFTERBURNER_JOB_ID={job_id}\nEnvironment=AFTERBURNER_LEASE_ID={lease_id}\nEnvironment=AFTERBURNER_GPU_IDS={gpu_id}\nExecStart={exec_start}\nKillSignal=SIGTERM\nTimeoutStopSec=30\n\n[Install]\nWantedBy=default.target\n",
        job_id = job.job_id,
        lease_id = lease_id,
        gpu_id = free_gpu.gpu_id,
        working_directory = job.working_directory,
        exec_start = job.exec_start,
    );

    if let Some(parent) = args.out_lease.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = args.out_unit.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &args.out_lease,
        serde_json::to_string_pretty(&lease).map_err(|err| {
            SingleNodeSchedulerError::Parse(format!("serialize lease json: {err}"))
        })?,
    )?;
    fs::write(&args.out_unit, unit)?;

    emit_event(
        "info",
        "deploy_cli",
        "single_node_gpu_lease_written",
        json!({
            "lease_path": args.out_lease.display().to_string(),
            "unit_path": args.out_unit.display().to_string(),
            "lease": lease,
        }),
    );

    Ok(())
}

#[derive(Debug)]
struct JobRequest {
    job_id: String,
    priority: u64,
    gpu_count: u64,
    checkpoint_mode: String,
    working_directory: String,
    exec_start: String,
}

#[derive(Debug)]
struct Inventory {
    node_id: String,
    gpus: Vec<GpuEntry>,
}

#[derive(Debug)]
struct GpuEntry {
    gpu_id: String,
    state: String,
}

fn load_job(path: &Path) -> Result<JobRequest, SingleNodeSchedulerError> {
    let value = load_json(path)?;
    let object = value.as_object().ok_or_else(|| {
        SingleNodeSchedulerError::Parse("job request must be an object".to_string())
    })?;
    expect_const(object, "schema_version", "1", path)?;
    let job_id = read_string(object, "job_id", path)?;
    let priority = read_u64(object, "priority", path)?;
    let gpu_count = read_u64(object, "gpu_count", path)?;
    let checkpoint_mode = read_string(object, "checkpoint_mode", path)?;
    if checkpoint_mode != "cooperative" && checkpoint_mode != "none" {
        return Err(SingleNodeSchedulerError::Parse(format!(
            "{} field `checkpoint_mode` must be `cooperative` or `none`",
            path.display()
        )));
    }
    let working_directory = read_string(object, "working_directory", path)?;
    let exec_start = read_string(object, "exec_start", path)?;
    Ok(JobRequest {
        job_id,
        priority,
        gpu_count,
        checkpoint_mode,
        working_directory,
        exec_start,
    })
}

fn load_inventory(path: &Path) -> Result<Inventory, SingleNodeSchedulerError> {
    let value = load_json(path)?;
    let object = value.as_object().ok_or_else(|| {
        SingleNodeSchedulerError::Parse("gpu inventory must be an object".to_string())
    })?;
    expect_const(object, "schema_version", "1", path)?;
    let node_id = read_string(object, "node_id", path)?;
    let gpus = object
        .get("gpus")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            SingleNodeSchedulerError::Parse(format!(
                "{} must define array field `gpus`",
                path.display()
            ))
        })?;
    let gpus = gpus
        .iter()
        .map(|value| {
            let object = value.as_object().ok_or_else(|| {
                SingleNodeSchedulerError::Parse(format!(
                    "{} gpu entries must be objects",
                    path.display()
                ))
            })?;
            Ok(GpuEntry {
                gpu_id: read_string(object, "gpu_id", path)?,
                state: read_string(object, "state", path)?,
            })
        })
        .collect::<Result<Vec<_>, SingleNodeSchedulerError>>()?;
    Ok(Inventory { node_id, gpus })
}

fn load_json(path: &Path) -> Result<Value, SingleNodeSchedulerError> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|err| SingleNodeSchedulerError::Parse(format!("parse {}: {err}", path.display())))
}

fn expect_const(
    object: &serde_json::Map<String, Value>,
    key: &str,
    expected: &str,
    path: &Path,
) -> Result<(), SingleNodeSchedulerError> {
    let actual = read_string(object, key, path)?;
    if actual != expected {
        return Err(SingleNodeSchedulerError::Parse(format!(
            "{} field `{key}` must be `{expected}`, got `{actual}`",
            path.display()
        )));
    }
    Ok(())
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<String, SingleNodeSchedulerError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| {
            SingleNodeSchedulerError::Parse(format!(
                "{} must define non-empty string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<u64, SingleNodeSchedulerError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        SingleNodeSchedulerError::Parse(format!(
            "{} must define non-negative integer field `{key}`",
            path.display()
        ))
    })
}

fn parse_args<I>(args: I) -> Result<Args, SingleNodeSchedulerError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut job_path = None::<PathBuf>;
    let mut inventory_path = None::<PathBuf>;
    let mut out_lease = PathBuf::from(DEFAULT_LEASE_PATH);
    let mut out_unit = PathBuf::from("artifacts/deploy/afterburner-job.service");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--job" => {
                job_path = Some(PathBuf::from(parse_value(&mut args, "--job")?));
            }
            "--inventory" => {
                inventory_path = Some(PathBuf::from(parse_value(&mut args, "--inventory")?));
            }
            "--out-lease" => {
                out_lease = PathBuf::from(parse_value(&mut args, "--out-lease")?);
            }
            "--out-unit" => {
                out_unit = PathBuf::from(parse_value(&mut args, "--out-unit")?);
            }
            _ if arg.starts_with("--job=") => {
                job_path = Some(PathBuf::from(arg.trim_start_matches("--job=")));
            }
            _ if arg.starts_with("--inventory=") => {
                inventory_path = Some(PathBuf::from(arg.trim_start_matches("--inventory=")));
            }
            _ if arg.starts_with("--out-lease=") => {
                out_lease = PathBuf::from(arg.trim_start_matches("--out-lease="));
            }
            _ if arg.starts_with("--out-unit=") => {
                out_unit = PathBuf::from(arg.trim_start_matches("--out-unit="));
            }
            _ => {
                return Err(SingleNodeSchedulerError::InvalidArg(format!(
                    "unknown argument for deploy single-node-scheduler: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        job_path: job_path.ok_or_else(|| {
            SingleNodeSchedulerError::InvalidArg(format!("missing value for --job\n{}", usage()))
        })?,
        inventory_path: inventory_path.ok_or_else(|| {
            SingleNodeSchedulerError::InvalidArg(format!(
                "missing value for --inventory\n{}",
                usage()
            ))
        })?,
        out_lease,
        out_unit,
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, SingleNodeSchedulerError>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| {
        SingleNodeSchedulerError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner deploy single-node-scheduler --job PATH --inventory PATH [--out-lease PATH] [--out-unit PATH]"
}

#[cfg(test)]
mod tests {
    use super::{Args, parse_args};
    use std::path::PathBuf;

    #[test]
    fn parse_args_accepts_required_flags() {
        let args = vec![
            "--job".to_string(),
            "fixtures/gpu_scheduler_job_request.example.json".to_string(),
            "--inventory".to_string(),
            "fixtures/gpu_inventory.example.json".to_string(),
        ];
        let parsed = parse_args(args.into_iter()).expect("parse scheduler args");
        assert_eq!(
            parsed,
            Args {
                job_path: PathBuf::from("fixtures/gpu_scheduler_job_request.example.json"),
                inventory_path: PathBuf::from("fixtures/gpu_inventory.example.json"),
                out_lease: PathBuf::from("artifacts/deploy/gpu_scheduler_lease.json"),
                out_unit: PathBuf::from("artifacts/deploy/afterburner-job.service"),
            }
        );
    }
}
