use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const PRETRAINING_SOURCE_PROVENANCE_EVIDENCE_BUNDLE_PATH: &str =
    "artifacts/data/pretraining_source_provenance_evidence_bundle.json";

#[derive(Debug)]
enum PretrainingSourceProvenanceEvidenceBundleError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for PretrainingSourceProvenanceEvidenceBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PretrainingSourceProvenanceEvidenceBundleError {}

impl From<std::io::Error> for PretrainingSourceProvenanceEvidenceBundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for PretrainingSourceProvenanceEvidenceBundleError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize pretraining source provenance evidence bundle json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    provenance_receipt: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), PretrainingSourceProvenanceEvidenceBundleError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let provenance_receipt = load_json_object(
        args.provenance_receipt.as_path(),
        "pretraining source provenance receipt",
    )?;
    let approval_receipt_path = read_string(
        &provenance_receipt,
        "approval_receipt_path",
        args.provenance_receipt.as_path(),
    )?;
    let approval_receipt = load_json_object(
        Path::new(approval_receipt_path.as_str()),
        "pretraining source approval receipt",
    )?;

    let bundle = json!({
        "schema_version": "1",
        "approval_receipt_path": approval_receipt_path,
        "provenance_receipt_path": args.provenance_receipt.display().to_string(),
        "source": read_string(&approval_receipt, "source", Path::new(approval_receipt_path.as_str()))?,
        "source_revision": read_string(&approval_receipt, "source_revision", Path::new(approval_receipt_path.as_str()))?,
        "approval_status": read_string(&approval_receipt, "approval_status", Path::new(approval_receipt_path.as_str()))?,
        "evidence_count": 1,
        "evidence_sources": [
            {
                "registry_entry_path": read_string(&provenance_receipt, "registry_entry_path", args.provenance_receipt.as_path())?,
                "upstream_locator": read_string(&provenance_receipt, "upstream_locator", args.provenance_receipt.as_path())?,
                "reviewed_metadata_sha256": read_string(&provenance_receipt, "reviewed_metadata_sha256", args.provenance_receipt.as_path())?
            }
        ]
    });

    write_json_value(&args.out_path, &bundle)?;
    emit_json_artifact_written(
        "data_cli",
        "pretraining_source_provenance_evidence_bundle_written",
        "bundle_path",
        &args.out_path,
        "bundle",
        bundle,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, PretrainingSourceProvenanceEvidenceBundleError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut provenance_receipt = None::<PathBuf>;
    let mut out_path = PathBuf::from(PRETRAINING_SOURCE_PROVENANCE_EVIDENCE_BUNDLE_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--provenance-receipt" => {
                let value: String = parse_value(&mut args, "--provenance-receipt")?;
                provenance_receipt = Some(PathBuf::from(value));
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--provenance-receipt=") => {
                provenance_receipt = Some(PathBuf::from(
                    arg.trim_start_matches("--provenance-receipt=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(PretrainingSourceProvenanceEvidenceBundleError::InvalidArg(
                    format!(
                        "unknown argument for source provenance-evidence-bundle: {arg}\n{}",
                        usage()
                    ),
                ));
            }
        }
    }

    Ok(Args {
        provenance_receipt: provenance_receipt.ok_or_else(|| {
            PretrainingSourceProvenanceEvidenceBundleError::InvalidArg(format!(
                "missing value for --provenance-receipt\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(
    args: &mut I,
    flag: &str,
) -> Result<T, PretrainingSourceProvenanceEvidenceBundleError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        PretrainingSourceProvenanceEvidenceBundleError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        PretrainingSourceProvenanceEvidenceBundleError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<serde_json::Map<String, Value>, PretrainingSourceProvenanceEvidenceBundleError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        PretrainingSourceProvenanceEvidenceBundleError::Parse(format!(
            "parse {kind} `{}`: {err}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        PretrainingSourceProvenanceEvidenceBundleError::Parse(format!(
            "{kind} `{}` must be an object",
            path.display()
        ))
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    path: &Path,
) -> Result<String, PretrainingSourceProvenanceEvidenceBundleError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            PretrainingSourceProvenanceEvidenceBundleError::Parse(format!(
                "pretraining source artifact `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn usage() -> &'static str {
    "usage: afterburner source provenance-evidence-bundle --provenance-receipt PATH [--out PATH]"
}
