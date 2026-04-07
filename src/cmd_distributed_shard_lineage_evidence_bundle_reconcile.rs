use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str =
    "artifacts/train/distributed_shard_lineage_evidence_bundle_reconciliation.json";

#[derive(Debug)]
enum DistributedShardLineageEvidenceBundleReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedShardLineageEvidenceBundleReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedShardLineageEvidenceBundleReconcileError {}

impl From<std::io::Error> for DistributedShardLineageEvidenceBundleReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DistributedShardLineageEvidenceBundleReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize distributed shard lineage evidence bundle reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    receipt: PathBuf,
    bundle: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DistributedShardLineageEvidenceBundleReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "distributed shard lineage receipt")?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "distributed shard lineage evidence bundle",
    )?;
    let evidence_sources = receipt.get("evidence_sources").cloned().ok_or_else(|| {
        DistributedShardLineageEvidenceBundleReconcileError::Parse(format!(
            "distributed shard lineage receipt `{}` missing field `evidence_sources`",
            args.receipt.display()
        ))
    })?;
    let evidence_count = evidence_sources
        .as_array()
        .map(|v| v.len())
        .ok_or_else(|| {
            DistributedShardLineageEvidenceBundleReconcileError::Parse(format!(
                "distributed shard lineage receipt `{}` field `evidence_sources` must be an array",
                args.receipt.display()
            ))
        })?;

    let desired = json!({
        "receipt_path": args.receipt.display().to_string(),
        "shard_id": read_string(&receipt, "shard_id", args.receipt.as_path(), "distributed shard lineage receipt")?,
        "source": read_string(&receipt, "source", args.receipt.as_path(), "distributed shard lineage receipt")?,
        "source_revision": read_string(&receipt, "source_revision", args.receipt.as_path(), "distributed shard lineage receipt")?,
        "checkpoint_group": read_string(&receipt, "checkpoint_group", args.receipt.as_path(), "distributed shard lineage receipt")?,
        "evidence_count": Value::from(evidence_count as u64),
        "evidence_sources": evidence_sources,
    });
    let current = json!({
        "receipt_path": read_string(&bundle, "receipt_path", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "shard_id": read_string(&bundle, "shard_id", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "source": read_string(&bundle, "source", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "source_revision": read_string(&bundle, "source_revision", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "checkpoint_group": read_string(&bundle, "checkpoint_group", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "evidence_sources": read_object(&bundle, "evidence_sources", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
    });
    let reconciliation_status = if desired == current {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "bundle_path": args.bundle.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "train_cli",
        "distributed_shard_lineage_evidence_bundle_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedShardLineageEvidenceBundleReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut bundle = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                receipt = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--receipt",
                )?))
            }
            "--bundle" => {
                bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--bundle",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--bundle=") => {
                bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DistributedShardLineageEvidenceBundleReconcileError::InvalidArg(format!(
                        "unknown argument for lineage bundle reconcile: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DistributedShardLineageEvidenceBundleReconcileError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        bundle: bundle.ok_or_else(|| {
            DistributedShardLineageEvidenceBundleReconcileError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DistributedShardLineageEvidenceBundleReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedShardLineageEvidenceBundleReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedShardLineageEvidenceBundleReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(
    DistributedShardLineageEvidenceBundleReconcileError
);

fn usage() -> &'static str {
    "usage: afterburner debug lineage bundle reconcile --receipt PATH --bundle PATH [--out PATH]"
}
