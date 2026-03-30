use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use afterburner::observability::{attach_trace_fields, current_trace_context, emit_event};
use serde_json::{Value, json};

const RECEIPT_SCHEMA_VERSION: &str = "1";
const DEFAULT_HF_BIN: &str = "hf";

#[derive(Debug)]
enum HfPublishError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
    Publish(String),
}

impl std::fmt::Display for HfPublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
            Self::Publish(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for HfPublishError {}

impl From<std::io::Error> for HfPublishError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    request_path: PathBuf,
    hf_bin: PathBuf,
    revision: String,
    out_path: Option<PathBuf>,
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

fn run_inner<I>(args: I) -> Result<PathBuf, HfPublishError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let trace = current_trace_context("deploy_cli");
    let request = load_request(&args.request_path)?;
    if request.provider != "huggingface" {
        return Err(HfPublishError::Publish(format!(
            "huggingface publish requires request.provider = `huggingface`, got `{}`",
            request.provider
        )));
    }

    let out_path = args.out_path.unwrap_or_else(|| {
        PathBuf::from("artifacts")
            .join("deploy")
            .join(request.artifact_version.as_str())
            .join("huggingface_publish_receipt.json")
    });

    let repo_id = request.destination.clone();
    let mut uploads = vec![
        UploadSpec {
            local_path: PathBuf::from(request.artifact_file.clone()),
            path_in_repo: format!("afterburner/{}/model.mpk", request.artifact_version),
        },
        UploadSpec {
            local_path: PathBuf::from(request.artifact_manifest.clone()),
            path_in_repo: format!("afterburner/{}/manifest.toml", request.artifact_version),
        },
        UploadSpec {
            local_path: args.request_path.clone(),
            path_in_repo: format!(
                "afterburner/{}/artifact_upload_request.json",
                request.artifact_version
            ),
        },
    ];
    uploads.extend(
        request
            .artifact_support_files
            .iter()
            .map(|support| UploadSpec {
                local_path: PathBuf::from(support.local_path.clone()),
                path_in_repo: format!(
                    "afterburner/{}/{}",
                    request.artifact_version, support.path_in_artifact_directory
                ),
            }),
    );

    for upload in &uploads {
        run_hf_upload(
            &args.hf_bin,
            repo_id.as_str(),
            args.revision.as_str(),
            upload.local_path.as_path(),
            upload.path_in_repo.as_str(),
        )?;
    }

    let mut receipt = json!({
        "schema_version": RECEIPT_SCHEMA_VERSION,
        "repo_id": repo_id,
        "revision": args.revision,
        "request_path": args.request_path.display().to_string(),
        "artifact_version": request.artifact_version,
        "uploaded_files": uploads.iter().map(|upload| {
            json!({
                "local_path": upload.local_path.display().to_string(),
                "path_in_repo": upload.path_in_repo,
            })
        }).collect::<Vec<_>>(),
        "published_at_unix_ms": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    });
    attach_trace_fields(
        receipt.as_object_mut().expect("receipt must be an object"),
        &trace,
    );

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&receipt)
        .map_err(|err| HfPublishError::Parse(format!("serialize hf publish receipt: {err}")))?;
    fs::write(&out_path, text)?;

    emit_event(
        "info",
        "deploy_cli",
        "huggingface_publish_receipt_written",
        json!({
            "receipt_path": out_path.display().to_string(),
            "receipt": receipt,
        }),
    );

    Ok(out_path)
}

struct UploadSpec {
    local_path: PathBuf,
    path_in_repo: String,
}

fn run_hf_upload(
    hf_bin: &Path,
    repo_id: &str,
    revision: &str,
    local_path: &Path,
    path_in_repo: &str,
) -> Result<(), HfPublishError> {
    let output = Command::new(hf_bin)
        .arg("upload")
        .arg(repo_id)
        .arg(local_path)
        .arg(path_in_repo)
        .arg("--repo-type")
        .arg("model")
        .arg("--revision")
        .arg(revision)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(HfPublishError::Publish(format!(
        "hf upload failed for `{}` -> `{}`: {}",
        local_path.display(),
        path_in_repo,
        stderr.trim()
    )))
}

struct UploadRequest {
    provider: String,
    destination: String,
    artifact_version: String,
    artifact_manifest: String,
    artifact_file: String,
    artifact_support_files: Vec<SupportFile>,
}

struct SupportFile {
    local_path: String,
    path_in_artifact_directory: String,
}

fn load_request(path: &Path) -> Result<UploadRequest, HfPublishError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| HfPublishError::Parse(format!("parse {}: {err}", path.display())))?;
    let object = value
        .as_object()
        .ok_or_else(|| HfPublishError::Parse("upload request must be a json object".to_string()))?;
    read_const_string(object, "schema_version", "1")?;
    read_const_string(object, "operation", "upload")?;

    Ok(UploadRequest {
        provider: read_string(object, "provider")?,
        destination: read_string(object, "destination")?,
        artifact_version: read_string(object, "artifact_version")?,
        artifact_manifest: read_string(object, "artifact_manifest")?,
        artifact_file: read_string(object, "artifact_file")?,
        artifact_support_files: read_support_files(object)?,
    })
}

fn read_support_files(
    object: &serde_json::Map<String, Value>,
) -> Result<Vec<SupportFile>, HfPublishError> {
    let Some(value) = object.get("artifact_support_files") else {
        return Ok(Vec::new());
    };
    let array = value.as_array().ok_or_else(|| {
        HfPublishError::Parse("field `artifact_support_files` must be an array".to_string())
    })?;
    array
        .iter()
        .map(|entry| {
            let entry = entry.as_object().ok_or_else(|| {
                HfPublishError::Parse(
                    "artifact_support_files entries must be json objects".to_string(),
                )
            })?;
            Ok(SupportFile {
                local_path: read_string(entry, "local_path")?,
                path_in_artifact_directory: read_string(entry, "path_in_artifact_directory")?,
            })
        })
        .collect()
}

fn read_const_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    expected: &str,
) -> Result<(), HfPublishError> {
    let actual = read_string(object, key)?;
    if actual != expected {
        return Err(HfPublishError::Parse(format!(
            "field `{key}` must be `{expected}`, got `{actual}`"
        )));
    }
    Ok(())
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<String, HfPublishError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| HfPublishError::Parse(format!("field `{key}` must be a non-empty string")))
}

fn parse_args<I>(args: I) -> Result<Args, HfPublishError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut request_path = None::<PathBuf>;
    let mut hf_bin = PathBuf::from(DEFAULT_HF_BIN);
    let mut revision = "main".to_string();
    let mut out_path = None::<PathBuf>;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--request" => request_path = Some(PathBuf::from(parse_value(&mut args, "--request")?)),
            "--hf-bin" => hf_bin = PathBuf::from(parse_value(&mut args, "--hf-bin")?),
            "--revision" => revision = parse_value(&mut args, "--revision")?,
            "--out" => out_path = Some(PathBuf::from(parse_value(&mut args, "--out")?)),
            _ if arg.starts_with("--request=") => {
                request_path = Some(PathBuf::from(arg.trim_start_matches("--request=")))
            }
            _ if arg.starts_with("--hf-bin=") => {
                hf_bin = PathBuf::from(arg.trim_start_matches("--hf-bin="))
            }
            _ if arg.starts_with("--revision=") => {
                revision = arg.trim_start_matches("--revision=").to_string()
            }
            _ if arg.starts_with("--out=") => {
                out_path = Some(PathBuf::from(arg.trim_start_matches("--out=")))
            }
            _ => {
                return Err(HfPublishError::InvalidArg(format!(
                    "unknown argument for deploy hf-publish: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        request_path: request_path.ok_or_else(|| {
            HfPublishError::InvalidArg(format!("missing value for --request\n{}", usage()))
        })?,
        hf_bin,
        revision,
        out_path,
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, HfPublishError>
where
    I: Iterator<Item = String>,
{
    args.next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| HfPublishError::InvalidArg(format!("missing value for {flag}\n{}", usage())))
}

fn usage() -> &'static str {
    "usage: afterburner deploy hf-publish --request PATH [--hf-bin PATH] [--revision NAME] [--out PATH]"
}

#[cfg(test)]
mod tests {
    use super::{Args, parse_args};
    use std::path::PathBuf;

    #[test]
    fn parse_args_accepts_required_flags() {
        let args = vec![
            "--request".to_string(),
            "artifacts/deploy/candidate_upload_request.json".to_string(),
            "--revision".to_string(),
            "main".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse args");
        assert_eq!(
            parsed,
            Args {
                request_path: PathBuf::from("artifacts/deploy/candidate_upload_request.json"),
                hf_bin: PathBuf::from("hf"),
                revision: "main".to_string(),
                out_path: None,
            }
        );
    }
}
