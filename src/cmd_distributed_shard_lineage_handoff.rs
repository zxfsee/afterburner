use std::path::PathBuf;

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::json;

const DISTRIBUTED_SHARD_LINEAGE_EVIDENCE_HANDOFF_PATH: &str =
    "artifacts/train/distributed_shard_lineage_evidence_handoff.json";

#[derive(Debug)]
enum DistributedShardLineageHandoffError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for DistributedShardLineageHandoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DistributedShardLineageHandoffError {}

impl From<std::io::Error> for DistributedShardLineageHandoffError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for DistributedShardLineageHandoffError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize distributed shard lineage evidence handoff json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
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

fn run_inner<I>(args: I) -> Result<(), DistributedShardLineageHandoffError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let bundle = load_json_object(
        args.bundle.as_path(),
        "distributed shard lineage evidence bundle",
    )?;

    let handoff = json!({
        "schema_version": "1",
        "bundle_path": args.bundle.display().to_string(),
        "shard_id": read_string(&bundle, "shard_id", args.bundle.as_path())?,
        "source": read_string(&bundle, "source", args.bundle.as_path())?,
        "source_revision": read_string(&bundle, "source_revision", args.bundle.as_path())?,
        "checkpoint_group": read_string(&bundle, "checkpoint_group", args.bundle.as_path())?,
        "evidence_count": read_u64(&bundle, "evidence_count", args.bundle.as_path())?
    });

    write_json_value(&args.out_path, &handoff)?;
    emit_json_artifact_written(
        "train_cli",
        "distributed_shard_lineage_evidence_handoff_written",
        "handoff_path",
        &args.out_path,
        "handoff",
        handoff,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, DistributedShardLineageHandoffError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut bundle = None::<PathBuf>;
    let mut out_path = PathBuf::from(DISTRIBUTED_SHARD_LINEAGE_EVIDENCE_HANDOFF_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bundle" => {
                let value: String = parse_value(&mut args, "--bundle")?;
                bundle = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
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
                return Err(DistributedShardLineageHandoffError::InvalidArg(format!(
                    "unknown argument for lineage handoff: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        bundle: bundle.ok_or_else(|| {
            DistributedShardLineageHandoffError::InvalidArg(format!(
                "missing value for --bundle\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, DistributedShardLineageHandoffError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        DistributedShardLineageHandoffError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        DistributedShardLineageHandoffError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

afterburner::define_command_input_json_kind_helpers!(
    DistributedShardLineageHandoffError,
    "distributed shard lineage evidence bundle"
);

fn usage() -> &'static str {
    "usage: afterburner debug lineage handoff --bundle PATH [--out PATH]"
}
