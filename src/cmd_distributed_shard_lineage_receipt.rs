use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DISTRIBUTED_SHARD_LINEAGE_RECEIPT_PATH: &str =
    "artifacts/train/distributed_shard_lineage_receipt.json";

#[derive(Debug)]
enum DistributedShardLineageReceiptError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedShardLineageReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedShardLineageReceiptError {}

impl From<std::io::Error> for DistributedShardLineageReceiptError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DistributedShardLineageReceiptError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize distributed shard lineage receipt json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    metadata: PathBuf,
    shard_id: String,
    source: String,
    source_revision: String,
    checkpoint_group: String,
    checkpoint_root: PathBuf,
    checked_at_unix_ms: u64,
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

fn run_inner<I>(args: I) -> Result<(), DistributedShardLineageReceiptError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let metadata = load_json_object(args.metadata.as_path(), "distributed shard metadata")?;
    let metadata_shard_id = metadata
        .get("shard_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            DistributedShardLineageReceiptError::Parse(format!(
                "distributed shard metadata `{}` missing string field `shard_id`",
                args.metadata.display()
            ))
        })?;
    if metadata_shard_id != args.shard_id {
        return Err(DistributedShardLineageReceiptError::Parse(format!(
            "distributed shard metadata `{}` has shard_id `{metadata_shard_id}` but expected `{}`",
            args.metadata.display(),
            args.shard_id
        )));
    }

    let receipt = json!({
        "schema_version": "1",
        "shard_id": args.shard_id,
        "source": args.source,
        "source_revision": args.source_revision,
        "checkpoint_group": args.checkpoint_group,
        "checked_at_unix_ms": args.checked_at_unix_ms,
        "evidence_sources": [
            {
                "metadata_path": args.metadata.display().to_string(),
                "checkpoint_root": args.checkpoint_root.display().to_string(),
                "observed_at_unix_ms": args.checked_at_unix_ms
            }
        ]
    });

    write_json_value(&args.out_path, &receipt)?;
    emit_json_artifact_written(
        "train_cli",
        "distributed_shard_lineage_receipt_written",
        "receipt_path",
        &args.out_path,
        "receipt",
        receipt,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedShardLineageReceiptError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut metadata = None::<PathBuf>;
    let mut shard_id = None::<String>;
    let mut source = None::<String>;
    let mut source_revision = None::<String>;
    let mut checkpoint_group = None::<String>;
    let mut checkpoint_root = None::<PathBuf>;
    let mut checked_at_unix_ms = None::<u64>;
    let mut out_path = PathBuf::from(DISTRIBUTED_SHARD_LINEAGE_RECEIPT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--metadata" => {
                metadata = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--metadata",
                )?))
            }
            "--shard-id" => shard_id = Some(parse_value(&mut args, "--shard-id")?),
            "--source" => source = Some(parse_value(&mut args, "--source")?),
            "--source-revision" => {
                source_revision = Some(parse_value(&mut args, "--source-revision")?)
            }
            "--checkpoint-group" => {
                checkpoint_group = Some(parse_value(&mut args, "--checkpoint-group")?)
            }
            "--checkpoint-root" => {
                checkpoint_root = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--checkpoint-root",
                )?))
            }
            "--checked-at-unix-ms" => {
                checked_at_unix_ms = Some(parse_value(&mut args, "--checked-at-unix-ms")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--metadata=") => {
                metadata = Some(PathBuf::from(
                    arg.trim_start_matches("--metadata=").to_string(),
                ))
            }
            _ if arg.starts_with("--shard-id=") => {
                shard_id = Some(arg.trim_start_matches("--shard-id=").to_string())
            }
            _ if arg.starts_with("--source=") => {
                source = Some(arg.trim_start_matches("--source=").to_string())
            }
            _ if arg.starts_with("--source-revision=") => {
                source_revision = Some(arg.trim_start_matches("--source-revision=").to_string())
            }
            _ if arg.starts_with("--checkpoint-group=") => {
                checkpoint_group = Some(arg.trim_start_matches("--checkpoint-group=").to_string())
            }
            _ if arg.starts_with("--checkpoint-root=") => {
                checkpoint_root = Some(PathBuf::from(
                    arg.trim_start_matches("--checkpoint-root=").to_string(),
                ))
            }
            _ if arg.starts_with("--checked-at-unix-ms=") => {
                checked_at_unix_ms = Some(
                    arg.trim_start_matches("--checked-at-unix-ms=")
                        .parse::<u64>()
                        .map_err(|_| {
                            DistributedShardLineageReceiptError::InvalidArg(format!(
                                "invalid value for --checked-at-unix-ms\n{}",
                                usage()
                            ))
                        })?,
                )
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DistributedShardLineageReceiptError::InvalidArg(format!(
                    "unknown argument for lineage receipt: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        metadata: metadata.ok_or_else(|| {
            DistributedShardLineageReceiptError::InvalidArg(format!(
                "missing value for --metadata\n{}",
                usage()
            ))
        })?,
        shard_id: require_non_empty(shard_id, "--shard-id")?,
        source: require_non_empty(source, "--source")?,
        source_revision: require_non_empty(source_revision, "--source-revision")?,
        checkpoint_group: require_non_empty(checkpoint_group, "--checkpoint-group")?,
        checkpoint_root: checkpoint_root.ok_or_else(|| {
            DistributedShardLineageReceiptError::InvalidArg(format!(
                "missing value for --checkpoint-root\n{}",
                usage()
            ))
        })?,
        checked_at_unix_ms: checked_at_unix_ms.ok_or_else(|| {
            DistributedShardLineageReceiptError::InvalidArg(format!(
                "missing value for --checked-at-unix-ms\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DistributedShardLineageReceiptError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedShardLineageReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedShardLineageReceiptError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn require_non_empty(
    value: Option<String>,
    flag: &str,
) -> Result<String, DistributedShardLineageReceiptError> {
    let value = value.ok_or_else(|| {
        DistributedShardLineageReceiptError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    if value.trim().is_empty() {
        return Err(DistributedShardLineageReceiptError::InvalidArg(format!(
            "{flag} must be non-empty\n{}",
            usage()
        )));
    }
    Ok(value)
}

afterburner::define_command_input_json_helpers!(DistributedShardLineageReceiptError);

fn usage() -> &'static str {
    "usage: afterburner lineage receipt --metadata PATH --shard-id ID --source NAME --source-revision REV --checkpoint-group NAME --checkpoint-root PATH --checked-at-unix-ms N [--out PATH]"
}
