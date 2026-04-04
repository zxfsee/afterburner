use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/train/distributed_shard_lineage_evidence_handoff_reconciliation.json";

#[derive(Debug)]
enum DistributedShardLineageHandoffReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedShardLineageHandoffReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedShardLineageHandoffReconcileError {}

impl From<std::io::Error> for DistributedShardLineageHandoffReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DistributedShardLineageHandoffReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize distributed shard lineage handoff reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    bundle: PathBuf,
    handoff: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DistributedShardLineageHandoffReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "distributed shard lineage evidence bundle",
    )?;
    let handoff = load_json_object(
        args.handoff.as_path(),
        "distributed shard lineage evidence handoff",
    )?;

    let desired = json!({
        "shard_id": read_string(&bundle, "shard_id", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "source": read_string(&bundle, "source", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "source_revision": read_string(&bundle, "source_revision", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "checkpoint_group": read_string(&bundle, "checkpoint_group", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path(), "distributed shard lineage evidence bundle")?,
    });
    let current = json!({
        "bundle_path": read_string(&handoff, "bundle_path", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "shard_id": read_string(&handoff, "shard_id", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "source": read_string(&handoff, "source", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "source_revision": read_string(&handoff, "source_revision", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "checkpoint_group": read_string(&handoff, "checkpoint_group", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "evidence_count": read_u64(&handoff, "evidence_count", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
    });
    let reconciliation_status = if desired["shard_id"] == current["shard_id"]
        && desired["source"] == current["source"]
        && desired["source_revision"] == current["source_revision"]
        && desired["checkpoint_group"] == current["checkpoint_group"]
        && desired["evidence_count"] == current["evidence_count"]
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "bundle_path": args.bundle.display().to_string(),
        "handoff_path": args.handoff.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "train_cli",
        "distributed_shard_lineage_evidence_handoff_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedShardLineageHandoffReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut bundle = None::<PathBuf>;
    let mut handoff = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bundle" => {
                bundle = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args, "--bundle",
                )?))
            }
            "--handoff" => {
                handoff = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--handoff",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--bundle=") => {
                bundle = Some(PathBuf::from(
                    arg.trim_start_matches("--bundle=").to_string(),
                ))
            }
            _ if arg.starts_with("--handoff=") => {
                handoff = Some(PathBuf::from(
                    arg.trim_start_matches("--handoff=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DistributedShardLineageHandoffReconcileError::InvalidArg(
                    format!(
                        "unknown argument for lineage handoff reconcile: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        bundle: bundle.ok_or_else(|| {
            DistributedShardLineageHandoffReconcileError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        handoff: handoff.ok_or_else(|| {
            DistributedShardLineageHandoffReconcileError::InvalidArg(format!(
                "missing value for --handoff\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DistributedShardLineageHandoffReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedShardLineageHandoffReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedShardLineageHandoffReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(DistributedShardLineageHandoffReconcileError);

fn usage() -> &'static str {
    "usage: afterburner lineage handoff reconcile --bundle PATH --handoff PATH [--out PATH]"
}
