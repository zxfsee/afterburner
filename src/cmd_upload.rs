use std::fs;
use std::path::{Path, PathBuf};

use afterburner::manifest::{ArtifactManifest, ManifestError};
use afterburner::observability::{attach_trace_fields, current_trace_context, emit_event};
use serde_json::{Value, json};

const UPLOAD_REQUEST_SCHEMA_VERSION: &str = "1";
const UPLOAD_OPERATION: &str = "upload";

#[derive(Debug)]
enum UploadError {
    InvalidArg(String),
    Io(std::io::Error),
    Manifest(ManifestError),
    Ownership(String),
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Manifest(err) => write!(f, "artifact manifest error: {err:?}"),
            Self::Ownership(err) => write!(f, "rollout ownership error: {err}"),
        }
    }
}

impl std::error::Error for UploadError {}

impl From<std::io::Error> for UploadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ManifestError> for UploadError {
    fn from(value: ManifestError) -> Self {
        Self::Manifest(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UploadArgs {
    manifest_path: PathBuf,
    ownership_path: PathBuf,
    package_contract_path: Option<PathBuf>,
    provider: String,
    destination: String,
    out_path: Option<PathBuf>,
}

struct OwnershipRecord {
    provenance_source: String,
    rollout_owner: String,
    approved_by: String,
    approval_ticket: String,
    approved_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SupportFile {
    role: String,
    local_path: String,
    path_in_artifact_directory: String,
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

fn run_inner<I>(args: I) -> Result<PathBuf, UploadError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let trace = current_trace_context("upload_cli");
    let manifest = ArtifactManifest::load_from_path(&args.manifest_path)?;
    let artifact_dir = args.manifest_path.parent().ok_or_else(|| {
        UploadError::InvalidArg("manifest path must have a parent directory".to_string())
    })?;
    let artifact_path = artifact_dir.join(&manifest.artifact);
    manifest.validate_against_current(&artifact_path)?;

    let ownership = load_ownership_record(&args.ownership_path)?;
    let support_files = if let Some(path) = args.package_contract_path.as_ref() {
        load_support_files(path, &manifest.artifact_version, &artifact_path)?
    } else {
        Vec::new()
    };
    let out_path = args.out_path.unwrap_or_else(|| {
        PathBuf::from("artifacts")
            .join("deploy")
            .join(manifest.artifact_version.as_str())
            .join("artifact_upload_request.json")
    });

    let provider = args.provider.clone();
    let destination = args.destination.clone();
    let mut request = json!({
        "schema_version": UPLOAD_REQUEST_SCHEMA_VERSION,
        "operation": UPLOAD_OPERATION,
        "provider": provider,
        "destination": destination,
        "artifact_version": manifest.artifact_version,
        "artifact_manifest": args.manifest_path.display().to_string(),
        "artifact_file": artifact_path.display().to_string(),
        "artifact_directory": artifact_dir.display().to_string(),
        "artifact_sha256": manifest.artifact_sha256,
        "provenance_source": ownership.provenance_source,
        "rollout_owner": ownership.rollout_owner,
        "approved_by": ownership.approved_by,
        "approval_ticket": ownership.approval_ticket,
        "approved_at_unix_ms": ownership.approved_at_unix_ms
    });
    if !support_files.is_empty() {
        request
            .as_object_mut()
            .expect("upload request must be an object")
            .insert(
                "artifact_support_files".to_string(),
                json!(
                    support_files
                        .iter()
                        .map(|file| {
                            json!({
                                "role": file.role,
                                "local_path": file.local_path,
                                "path_in_artifact_directory": file.path_in_artifact_directory,
                            })
                        })
                        .collect::<Vec<_>>()
                ),
            );
    }
    attach_trace_fields(
        request
            .as_object_mut()
            .expect("upload request must be an object"),
        &trace,
    );
    let request = serde_json::to_string_pretty(&request).map_err(|err| {
        UploadError::InvalidArg(format!("failed to serialize upload request json: {err}"))
    })?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, request)?;
    emit_event(
        "info",
        "deploy_cli",
        "artifact_upload_request_written",
        json!({
            "request_path": out_path.display().to_string(),
            "artifact_version": manifest.artifact_version,
            "provider": args.provider,
            "destination": args.destination,
            "traceparent": trace.traceparent,
            "trace_id": trace.trace_id,
        }),
    );
    Ok(out_path)
}

fn parse_args<I>(args: I) -> Result<UploadArgs, UploadError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut manifest_path = None::<PathBuf>;
    let mut ownership_path = None::<PathBuf>;
    let mut package_contract_path = None::<PathBuf>;
    let mut provider = None::<String>;
    let mut destination = None::<String>;
    let mut out_path = None::<PathBuf>;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => {
                manifest_path = Some(PathBuf::from(parse_value(&mut args, "--manifest")?))
            }
            "--ownership" => {
                ownership_path = Some(PathBuf::from(parse_value(&mut args, "--ownership")?))
            }
            "--package-contract" => {
                package_contract_path =
                    Some(PathBuf::from(parse_value(&mut args, "--package-contract")?))
            }
            "--provider" => provider = Some(parse_value(&mut args, "--provider")?),
            "--destination" => destination = Some(parse_value(&mut args, "--destination")?),
            "--out" => out_path = Some(PathBuf::from(parse_value(&mut args, "--out")?)),
            _ if arg.starts_with("--manifest=") => {
                manifest_path = Some(PathBuf::from(parse_inline_value(
                    arg.trim_start_matches("--manifest="),
                    "--manifest",
                )?))
            }
            _ if arg.starts_with("--ownership=") => {
                ownership_path = Some(PathBuf::from(parse_inline_value(
                    arg.trim_start_matches("--ownership="),
                    "--ownership",
                )?))
            }
            _ if arg.starts_with("--package-contract=") => {
                package_contract_path = Some(PathBuf::from(parse_inline_value(
                    arg.trim_start_matches("--package-contract="),
                    "--package-contract",
                )?))
            }
            _ if arg.starts_with("--provider=") => {
                provider = Some(parse_inline_value(
                    arg.trim_start_matches("--provider="),
                    "--provider",
                )?)
            }
            _ if arg.starts_with("--destination=") => {
                destination = Some(parse_inline_value(
                    arg.trim_start_matches("--destination="),
                    "--destination",
                )?)
            }
            _ if arg.starts_with("--out=") => {
                out_path = Some(PathBuf::from(parse_inline_value(
                    arg.trim_start_matches("--out="),
                    "--out",
                )?))
            }
            _ => {
                return Err(UploadError::InvalidArg(format!(
                    "unknown argument for upload: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    let manifest_path = manifest_path.ok_or_else(|| {
        UploadError::InvalidArg(format!("missing value for --manifest\n{}", usage()))
    })?;
    let ownership_path = ownership_path.ok_or_else(|| {
        UploadError::InvalidArg(format!("missing value for --ownership\n{}", usage()))
    })?;
    let provider = provider.ok_or_else(|| {
        UploadError::InvalidArg(format!("missing value for --provider\n{}", usage()))
    })?;
    let destination = destination.ok_or_else(|| {
        UploadError::InvalidArg(format!("missing value for --destination\n{}", usage()))
    })?;

    Ok(UploadArgs {
        manifest_path,
        ownership_path,
        package_contract_path,
        provider,
        destination,
        out_path,
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, UploadError>
where
    I: Iterator<Item = String>,
{
    let value = args
        .next()
        .ok_or_else(|| UploadError::InvalidArg(format!("missing value for {flag}\n{}", usage())))?;
    parse_inline_value(value.as_str(), flag)
}

fn parse_inline_value(value: &str, flag: &str) -> Result<String, UploadError> {
    if value.trim().is_empty() {
        return Err(UploadError::InvalidArg(format!(
            "{flag} must be non-empty\n{}",
            usage()
        )));
    }
    Ok(value.to_string())
}

fn load_ownership_record(path: &Path) -> Result<OwnershipRecord, UploadError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| UploadError::Ownership(format!("failed to parse ownership json: {err}")))?;
    let object = value.as_object().ok_or_else(|| {
        UploadError::Ownership("ownership file must contain a json object".to_string())
    })?;

    let schema_version = read_string(object, "schema_version")?;
    if schema_version != "1" {
        return Err(UploadError::Ownership(format!(
            "schema_version must be `1`, found `{schema_version}`"
        )));
    }

    let approved_operations = object
        .get("approved_operations")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            UploadError::Ownership("approved_operations must be an array".to_string())
        })?;
    let upload_approved = approved_operations
        .iter()
        .any(|value| value.as_str() == Some(UPLOAD_OPERATION));
    if !upload_approved {
        return Err(UploadError::Ownership(
            "approved_operations must include `upload`".to_string(),
        ));
    }

    Ok(OwnershipRecord {
        provenance_source: read_string(object, "provenance_source")?,
        rollout_owner: read_string(object, "rollout_owner")?,
        approved_by: read_string(object, "approved_by")?,
        approval_ticket: read_string(object, "approval_ticket")?,
        approved_at_unix_ms: read_u64(object, "approved_at_unix_ms")?,
    })
}

fn read_string(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<String, UploadError> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| UploadError::Ownership(format!("{key} must be a string")))?;
    if value.is_empty() {
        return Err(UploadError::Ownership(format!("{key} must be non-empty")));
    }
    Ok(value.to_string())
}

fn read_u64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<u64, UploadError> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| UploadError::Ownership(format!("{key} must be a non-negative integer")))
}

fn load_support_files(
    package_contract_path: &Path,
    artifact_version: &str,
    artifact_path: &Path,
) -> Result<Vec<SupportFile>, UploadError> {
    let text = fs::read_to_string(package_contract_path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        UploadError::Ownership(format!("failed to parse package contract json: {err}"))
    })?;
    let object = value.as_object().ok_or_else(|| {
        UploadError::Ownership("package contract must contain a json object".to_string())
    })?;

    let package_version = read_string(object, "input_artifact_version")?;
    if package_version != artifact_version {
        return Err(UploadError::Ownership(format!(
            "package contract input_artifact_version `{package_version}` must match manifest artifact_version `{artifact_version}`"
        )));
    }

    let optimized_artifact_path = read_string(object, "optimized_artifact_path")?;
    if Path::new(&optimized_artifact_path) != artifact_path {
        return Err(UploadError::Ownership(format!(
            "package contract optimized_artifact_path `{optimized_artifact_path}` must match upload artifact `{}`",
            artifact_path.display()
        )));
    }

    let packaging_inputs = object
        .get("packaging_inputs")
        .and_then(Value::as_object)
        .ok_or_else(|| UploadError::Ownership("packaging_inputs must be an object".to_string()))?;
    let optimization_profile_path = read_string(packaging_inputs, "optimization_profile_path")?;
    let layout = object
        .get("package_layout")
        .and_then(Value::as_array)
        .ok_or_else(|| UploadError::Ownership("package_layout must be an array".to_string()))?;

    let mut support_files = Vec::new();
    for entry in layout {
        let entry = entry.as_object().ok_or_else(|| {
            UploadError::Ownership("package_layout entries must be objects".to_string())
        })?;
        let role = read_string(entry, "role")?;
        let relative_path = read_string(entry, "path")?;
        match role.as_str() {
            "optimization_profile" => support_files.push(SupportFile {
                role,
                local_path: optimization_profile_path.clone(),
                path_in_artifact_directory: relative_path,
            }),
            "package_contract" => support_files.push(SupportFile {
                role,
                local_path: package_contract_path.display().to_string(),
                path_in_artifact_directory: relative_path,
            }),
            _ => {}
        }
    }

    Ok(support_files)
}

fn usage() -> &'static str {
    "usage: afterburner deploy upload --manifest PATH --ownership PATH [--package-contract PATH] --provider NAME --destination REF [--out PATH]"
}

#[cfg(test)]
mod tests {
    use super::{UploadArgs, parse_args};
    use std::path::PathBuf;

    #[test]
    fn parse_args_accepts_required_upload_contract_flags() {
        let args = vec![
            "--manifest".to_string(),
            "artifacts/inference/0.1.0/manifest.toml".to_string(),
            "--ownership=artifacts/deploy/ownership.json".to_string(),
            "--provider".to_string(),
            "generic".to_string(),
            "--destination=uploads/afterburner".to_string(),
            "--out".to_string(),
            "artifacts/deploy/out.json".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse upload args");
        assert_eq!(
            parsed,
            UploadArgs {
                manifest_path: PathBuf::from("artifacts/inference/0.1.0/manifest.toml"),
                ownership_path: PathBuf::from("artifacts/deploy/ownership.json"),
                package_contract_path: None,
                provider: "generic".to_string(),
                destination: "uploads/afterburner".to_string(),
                out_path: Some(PathBuf::from("artifacts/deploy/out.json")),
            }
        );
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let args = vec!["--bogus".to_string()];
        let err = parse_args(args.into_iter()).expect_err("unknown upload flag must fail");
        assert!(
            err.to_string()
                .contains("unknown argument for upload: --bogus")
        );
    }
}
