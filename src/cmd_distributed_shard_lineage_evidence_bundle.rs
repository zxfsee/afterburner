use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DISTRIBUTED_SHARD_LINEAGE_EVIDENCE_BUNDLE_PATH: &str =
    "artifacts/train/distributed_shard_lineage_evidence_bundle.json";

#[derive(Debug)]
enum DistributedShardLineageEvidenceBundleError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedShardLineageEvidenceBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedShardLineageEvidenceBundleError {}

impl From<std::io::Error> for DistributedShardLineageEvidenceBundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DistributedShardLineageEvidenceBundleError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize distributed shard lineage evidence bundle json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    receipt: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), DistributedShardLineageEvidenceBundleError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let receipt = load_json_object(args.receipt.as_path(), "distributed shard lineage receipt")?;
    let evidence_sources = receipt
        .get("evidence_sources")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            DistributedShardLineageEvidenceBundleError::Parse(format!(
                "distributed shard lineage receipt `{}` missing array field `evidence_sources`",
                args.receipt.display()
            ))
        })?;

    for (idx, entry) in evidence_sources.iter().enumerate() {
        let object = entry.as_object().ok_or_else(|| {
            DistributedShardLineageEvidenceBundleError::Parse(format!(
                "distributed shard lineage receipt `{}` evidence_sources[{idx}] must be an object",
                args.receipt.display()
            ))
        })?;
        for key in ["metadata_path", "checkpoint_root"] {
            if object.get(key).and_then(Value::as_str).is_none() {
                return Err(DistributedShardLineageEvidenceBundleError::Parse(format!(
                    "distributed shard lineage receipt `{}` evidence_sources[{idx}] missing string field `{key}`",
                    args.receipt.display()
                )));
            }
        }
        if object
            .get("observed_at_unix_ms")
            .and_then(Value::as_i64)
            .is_none()
        {
            return Err(DistributedShardLineageEvidenceBundleError::Parse(format!(
                "distributed shard lineage receipt `{}` evidence_sources[{idx}] missing integer field `observed_at_unix_ms`",
                args.receipt.display()
            )));
        }
    }

    let bundle = json!({
        "schema_version": "1",
        "receipt_path": args.receipt.display().to_string(),
        "shard_id": read_string(&receipt, "shard_id", args.receipt.as_path())?,
        "source": read_string(&receipt, "source", args.receipt.as_path())?,
        "source_revision": read_string(&receipt, "source_revision", args.receipt.as_path())?,
        "checkpoint_group": read_string(&receipt, "checkpoint_group", args.receipt.as_path())?,
        "evidence_count": evidence_sources.len(),
        "evidence_sources": evidence_sources,
    });

    write_json_value(&args.out_path, &bundle)?;
    emit_json_artifact_written(
        "train_cli",
        "distributed_shard_lineage_evidence_bundle_written",
        "bundle_path",
        &args.out_path,
        "bundle",
        bundle,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedShardLineageEvidenceBundleError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut receipt = None::<PathBuf>;
    let mut out_path = PathBuf::from(DISTRIBUTED_SHARD_LINEAGE_EVIDENCE_BUNDLE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--receipt" => {
                let value: String = parse_value(&mut args, "--receipt")?;
                receipt = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(DistributedShardLineageEvidenceBundleError::InvalidArg(
                    format!(
                        "unknown argument for lineage evidence-bundle: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        receipt: receipt.ok_or_else(|| {
            DistributedShardLineageEvidenceBundleError::InvalidArg(format!(
                "missing value for --receipt\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, DistributedShardLineageEvidenceBundleError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedShardLineageEvidenceBundleError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedShardLineageEvidenceBundleError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, DistributedShardLineageEvidenceBundleError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        DistributedShardLineageEvidenceBundleError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        DistributedShardLineageEvidenceBundleError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, DistributedShardLineageEvidenceBundleError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            DistributedShardLineageEvidenceBundleError::Parse(format!(
                "distributed shard lineage receipt `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner lineage evidence-bundle --receipt PATH [--out PATH]"
}
