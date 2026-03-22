use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin_cmd;
use burn::{backend::ndarray::NdArray, prelude::*, record::CompactRecorder};

type CpuBackend = NdArray<f32>;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn write_runtime_model_artifact(artifact_dir: &Path) -> PathBuf {
    let weights_path = artifact_dir.join("model.mpk");

    let device = <CpuBackend as Backend>::Device::default();
    let model = afterburner::model::ModelConfig::new(10).init::<CpuBackend>(&device);
    model
        .save_file(&weights_path, &CompactRecorder::new())
        .expect("write model artifact");

    let checksum =
        afterburner::manifest::compute_sha256_hex(&weights_path).expect("compute model checksum");
    let manifest =
        afterburner::manifest::ArtifactManifest::for_current("model.mpk", "0.1.0", checksum);
    manifest
        .write_to_dir(artifact_dir)
        .expect("write model manifest");

    artifact_dir.join("manifest.toml")
}

#[test]
fn artifact_upload_request_schema_and_docs_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("artifact_upload_request.schema.json"))
        .expect("read upload request schema fixture");
    let schema: serde_json::Value =
        serde_json::from_str(&schema_text).expect("parse upload request schema");

    assert_eq!(
        schema.get("$id").and_then(|value| value.as_str()),
        Some("https://afterburner.local/schemas/artifact-upload-request/v1")
    );
    assert_eq!(
        schema.get("type").and_then(|value| value.as_str()),
        Some("object")
    );
    assert_eq!(
        schema
            .get("additionalProperties")
            .and_then(|value| value.as_bool()),
        Some(false)
    );

    let required = schema
        .get("required")
        .and_then(|value| value.as_array())
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "operation",
        "provider",
        "destination",
        "traceparent",
        "trace_id",
        "artifact_version",
        "artifact_manifest",
        "artifact_file",
        "artifact_directory",
        "artifact_sha256",
        "provenance_source",
        "rollout_owner",
        "approved_by",
        "approval_ticket",
        "approved_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-014"),
        "architecture decisions index must link ADR-014"
    );

    let adr = repo_file("docs/adr/014-artifact-upload-adapter.md");
    assert!(
        adr.contains("afterburner deploy upload"),
        "ADR-014 must describe the upload CLI adapter"
    );
    assert!(
        adr.contains("provider-neutral"),
        "ADR-014 must describe the provider-neutral upload contract"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("artifact_upload_request.json"),
        "README must mention the upload request artifact"
    );
}

#[test]
fn upload_writes_provider_neutral_request_artifact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let manifest_path = write_runtime_model_artifact(&artifact_dir);
    let ownership_path = tmp.path().join("artifact_rollout_ownership.json");
    fs::write(
        &ownership_path,
        r#"{"schema_version":"1","artifact_version":"0.1.0","artifact_manifest":"artifacts/inference/0.1.0/manifest.toml","provenance_source":"artifacts/train/train_done.jsonl","rollout_owner":"ml-release","approved_by":"ops-review","approved_operations":["upload","deploy"],"approval_ticket":"CHG-4242","approved_at_unix_ms":1735689600000}"#,
    )
    .expect("write ownership file");

    let out_path = tmp.path().join("artifact_upload_request.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("upload")
        .arg("--manifest")
        .arg(&manifest_path)
        .arg("--ownership")
        .arg(&ownership_path)
        .arg("--provider")
        .arg("generic")
        .arg("--destination")
        .arg("uploads/afterburner")
        .arg("--out")
        .arg(&out_path);
    cmd.assert().success().code(0);

    let output = fs::read_to_string(&out_path).expect("read upload request");
    let mut actual: serde_json::Value =
        serde_json::from_str(&output).expect("parse upload request json");
    normalize_upload_request(&mut actual);

    let expected_text = fs::read_to_string(fixture_path("artifact_upload_request.example.json"))
        .expect("read upload request example fixture");
    let expected: serde_json::Value =
        serde_json::from_str(&expected_text).expect("parse upload request example fixture");

    assert_eq!(
        actual, expected,
        "upload request artifact must match the documented provider-neutral contract"
    );
}

#[test]
fn upload_requires_upload_approval_scope() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let manifest_path = write_runtime_model_artifact(&artifact_dir);
    let ownership_path = tmp.path().join("artifact_rollout_ownership.json");
    fs::write(
        &ownership_path,
        r#"{"schema_version":"1","artifact_version":"0.1.0","artifact_manifest":"artifacts/inference/0.1.0/manifest.toml","provenance_source":"artifacts/train/train_done.jsonl","rollout_owner":"ml-release","approved_by":"ops-review","approved_operations":["deploy"],"approval_ticket":"CHG-4242","approved_at_unix_ms":1735689600000}"#,
    )
    .expect("write ownership file without upload approval");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("upload")
        .arg("--manifest")
        .arg(&manifest_path)
        .arg("--ownership")
        .arg(&ownership_path)
        .arg("--provider")
        .arg("generic")
        .arg("--destination")
        .arg("uploads/afterburner");
    let assert = cmd.assert().failure().code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("approved_operations must include `upload`"),
        "missing upload approval must fail with a clear error"
    );
}

fn normalize_upload_request(value: &mut serde_json::Value) {
    let object = value
        .as_object_mut()
        .expect("upload request must be represented as an object");
    object.insert(
        "traceparent".to_string(),
        serde_json::json!("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
    );
    object.insert(
        "trace_id".to_string(),
        serde_json::json!("4bf92f3577b34da6a3ce929d0e0e4736"),
    );
    object.insert(
        "artifact_manifest".to_string(),
        serde_json::json!("<artifact>/manifest.toml"),
    );
    object.insert(
        "artifact_file".to_string(),
        serde_json::json!("<artifact>/model.mpk"),
    );
    object.insert(
        "artifact_directory".to_string(),
        serde_json::json!("<artifact>"),
    );
    object.insert(
        "artifact_sha256".to_string(),
        serde_json::json!("<artifact-sha256>"),
    );
}
