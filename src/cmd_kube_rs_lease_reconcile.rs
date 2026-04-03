use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str = "artifacts/deploy/kube_rs_gpu_lease_reconciliation.json";

#[derive(Debug)]
enum KubeRsLeaseReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for KubeRsLeaseReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for KubeRsLeaseReconcileError {}

impl From<std::io::Error> for KubeRsLeaseReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for KubeRsLeaseReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize kube-rs lease reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    lease: PathBuf,
    namespace: String,
    resource_name: String,
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
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            2
        }
    }
}

fn run_inner<I>(args: I) -> Result<(), KubeRsLeaseReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let lease = load_json_object(args.lease.as_path(), "gpu scheduler lease")?;

    let reconciliation = json!({
        "schema_version": "1",
        "lease_path": args.lease.display().to_string(),
        "lease_id": read_string(&lease, "lease_id", args.lease.as_path())?,
        "job_id": read_string(&lease, "job_id", args.lease.as_path())?,
        "node_id": read_string(&lease, "node_id", args.lease.as_path())?,
        "gpu_ids": read_string_array(&lease, "gpu_ids", args.lease.as_path())?,
        "checkpoint_mode": read_string(&lease, "checkpoint_mode", args.lease.as_path())?,
        "namespace": args.namespace,
        "resource_name": args.resource_name,
        "desired_state": "present",
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "deploy_cli",
        "kube_rs_gpu_lease_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, KubeRsLeaseReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut lease = None::<PathBuf>;
    let mut namespace = None::<String>;
    let mut resource_name = None::<String>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lease" => {
                lease = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--lease",
                )?))
            }
            "--namespace" => namespace = Some(parse_value(&mut args, "--namespace")?),
            "--resource-name" => resource_name = Some(parse_value(&mut args, "--resource-name")?),
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--lease=") => {
                lease = Some(PathBuf::from(
                    arg.trim_start_matches("--lease=").to_string(),
                ))
            }
            _ if arg.starts_with("--namespace=") => {
                namespace = Some(arg.trim_start_matches("--namespace=").to_string())
            }
            _ if arg.starts_with("--resource-name=") => {
                resource_name = Some(arg.trim_start_matches("--resource-name=").to_string())
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(KubeRsLeaseReconcileError::InvalidArg(format!(
                    "unknown argument for deploy kube-rs-lease-reconcile: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        lease: lease.ok_or_else(|| {
            KubeRsLeaseReconcileError::InvalidArg(format!("missing value for --lease\n{}", usage()))
        })?,
        namespace: namespace.ok_or_else(|| {
            KubeRsLeaseReconcileError::InvalidArg(format!(
                "missing value for --namespace\n{}",
                usage()
            ))
        })?,
        resource_name: resource_name.ok_or_else(|| {
            KubeRsLeaseReconcileError::InvalidArg(format!(
                "missing value for --resource-name\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, KubeRsLeaseReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        KubeRsLeaseReconcileError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        KubeRsLeaseReconcileError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

afterburner::define_command_input_json_kind_helpers!(KubeRsLeaseReconcileError, "gpu scheduler lease");

fn usage() -> &'static str {
    "usage: afterburner deploy kube-rs-lease-reconcile --lease PATH --namespace NAME --resource-name NAME [--out PATH]"
}
