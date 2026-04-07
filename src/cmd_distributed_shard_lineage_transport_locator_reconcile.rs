use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DEFAULT_OUT_PATH: &str =
    "artifacts/train/distributed_shard_lineage_transport_locator_reconciliation.json";

#[derive(Debug)]
enum DistributedShardLineageTransportLocatorReconcileError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedShardLineageTransportLocatorReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedShardLineageTransportLocatorReconcileError {}

impl From<std::io::Error> for DistributedShardLineageTransportLocatorReconcileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DistributedShardLineageTransportLocatorReconcileError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize distributed shard lineage transport locator reconciliation json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    handoff: PathBuf,
    locator: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DistributedShardLineageTransportLocatorReconcileError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let handoff = load_json_object(
        args.handoff.as_path(),
        "distributed shard lineage evidence handoff",
    )?;
    let locator = load_json_object(
        args.locator.as_path(),
        "distributed shard lineage transport locator",
    )?;

    let desired = json!({
        "shard_id": read_string(&handoff, "shard_id", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "source": read_string(&handoff, "source", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "source_revision": read_string(&handoff, "source_revision", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
        "checkpoint_group": read_string(&handoff, "checkpoint_group", args.handoff.as_path(), "distributed shard lineage evidence handoff")?,
    });
    let current = json!({
        "handoff_path": read_string(&locator, "handoff_path", args.locator.as_path(), "distributed shard lineage transport locator")?,
        "shard_id": read_string(&locator, "shard_id", args.locator.as_path(), "distributed shard lineage transport locator")?,
        "source": read_string(&locator, "source", args.locator.as_path(), "distributed shard lineage transport locator")?,
        "source_revision": read_string(&locator, "source_revision", args.locator.as_path(), "distributed shard lineage transport locator")?,
        "checkpoint_group": read_string(&locator, "checkpoint_group", args.locator.as_path(), "distributed shard lineage transport locator")?,
    });
    let reconciliation_status = if desired["shard_id"] == current["shard_id"]
        && desired["source"] == current["source"]
        && desired["source_revision"] == current["source_revision"]
        && desired["checkpoint_group"] == current["checkpoint_group"]
    {
        "aligned"
    } else {
        "needs-update"
    };

    let reconciliation = json!({
        "schema_version": "1",
        "handoff_path": args.handoff.display().to_string(),
        "locator_path": args.locator.display().to_string(),
        "desired": desired,
        "current": current,
        "reconciliation_status": reconciliation_status,
    });

    write_json_value(&args.out_path, &reconciliation)?;
    emit_json_artifact_written(
        "train_cli",
        "distributed_shard_lineage_transport_locator_reconciliation_written",
        "reconciliation_path",
        &args.out_path,
        "reconciliation",
        reconciliation,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedShardLineageTransportLocatorReconcileError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut handoff = None::<PathBuf>;
    let mut locator = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--handoff" => {
                handoff = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--handoff",
                )?))
            }
            "--locator" => {
                locator = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--locator",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--handoff=") => {
                handoff = Some(PathBuf::from(
                    arg.trim_start_matches("--handoff=").to_string(),
                ))
            }
            _ if arg.starts_with("--locator=") => {
                locator = Some(PathBuf::from(
                    arg.trim_start_matches("--locator=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(
                    DistributedShardLineageTransportLocatorReconcileError::InvalidArg(format!(
                        "unknown argument for lineage locator transport reconcile: {arg}\n{}",
                        usage()
                    )),
                );
            }
        }
    }

    Ok(Args {
        handoff: handoff.ok_or_else(|| {
            DistributedShardLineageTransportLocatorReconcileError::InvalidArg(format!(
                "missing value for --handoff\n{}",
                usage()
            ))
        })?,
        locator: locator.ok_or_else(|| {
            DistributedShardLineageTransportLocatorReconcileError::InvalidArg(format!(
                "missing value for --locator\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DistributedShardLineageTransportLocatorReconcileError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedShardLineageTransportLocatorReconcileError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedShardLineageTransportLocatorReconcileError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_helpers!(
    DistributedShardLineageTransportLocatorReconcileError
);

fn usage() -> &'static str {
    "usage: afterburner debug lineage locator transport reconcile --handoff PATH --locator PATH [--out PATH]"
}
