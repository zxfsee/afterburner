use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin_cmd;
use burn::{backend::ndarray::NdArray, prelude::*, record::CompactRecorder};
use serde_json::Value;

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

fn write_fake_hf(path: &Path, log_path: &Path) {
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {}\n",
        log_path.display()
    );
    fs::write(path, script).expect("write fake hf");
    let mut perms = fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod fake hf");
}

#[test]
fn huggingface_publish_receipt_schema_and_docs_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("huggingface_publish_receipt.schema.json"))
        .expect("read huggingface publish receipt schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/huggingface-publish-receipt/v1")
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be array")
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
        "repo_id",
        "revision",
        "request_path",
        "traceparent",
        "trace_id",
        "artifact_version",
        "uploaded_files",
        "published_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-055"),
        "architecture decisions index must link ADR-055"
    );

    let adr = repo_file("docs/adr/055-hugging-face-publish-adapter.md");
    for needle in [
        "deploy hf-publish",
        "provider-neutral upload request contract",
        "hf upload",
        "huggingface_publish_receipt.json",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-055 must mention `{needle}` as part of the Hugging Face adapter decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("deploy hf-publish"),
        "workflow reference must mention the Hugging Face publish command"
    );
}

#[test]
fn hf_publish_shells_out_and_writes_receipt() {
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

    let request_path = tmp.path().join("artifact_upload_request.json");
    let mut upload_cmd = cargo_bin_cmd!("afterburner");
    upload_cmd
        .arg("deploy")
        .arg("upload")
        .arg("--manifest")
        .arg(&manifest_path)
        .arg("--ownership")
        .arg(&ownership_path)
        .arg("--provider")
        .arg("huggingface")
        .arg("--destination")
        .arg("afterburner/model")
        .arg("--out")
        .arg(&request_path);
    upload_cmd.assert().success().code(0);

    let fake_hf_path = tmp.path().join("hf");
    let hf_log = tmp.path().join("hf.log");
    write_fake_hf(fake_hf_path.as_path(), hf_log.as_path());

    let receipt_path = tmp.path().join("huggingface_publish_receipt.json");
    let mut publish_cmd = cargo_bin_cmd!("afterburner");
    publish_cmd
        .arg("deploy")
        .arg("hf-publish")
        .arg("--request")
        .arg(&request_path)
        .env(
            "AFTERBURNER_TRACEPARENT",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        )
        .arg("--hf-bin")
        .arg(&fake_hf_path)
        .arg("--revision")
        .arg("main")
        .arg("--out")
        .arg(&receipt_path);
    let assert = publish_cmd.assert().success();

    let log_text = fs::read_to_string(&hf_log).expect("read hf log");
    let log_lines = log_text.lines().collect::<Vec<_>>();
    assert_eq!(
        log_lines.len(),
        3,
        "must upload artifact, manifest, and request"
    );
    assert!(log_lines[0].contains("upload afterburner/model"));
    assert!(log_lines[1].contains("afterburner/0.1.0/manifest.toml"));
    assert!(log_lines[2].contains("afterburner/0.1.0/artifact_upload_request.json"));

    let receipt_text = fs::read_to_string(&receipt_path).expect("read receipt");
    let mut receipt: Value = serde_json::from_str(&receipt_text).expect("parse receipt");
    let receipt_object = receipt
        .as_object_mut()
        .expect("receipt must be represented as object");
    receipt_object.insert("request_path".to_string(), Value::from("<request>"));
    receipt_object.insert(
        "published_at_unix_ms".to_string(),
        Value::from(1735689600000_u64),
    );
    let uploaded_files = receipt_object
        .get_mut("uploaded_files")
        .and_then(Value::as_array_mut)
        .expect("uploaded_files must be array");
    uploaded_files[0] = serde_json::json!({
        "local_path": "<artifact>/model.mpk",
        "path_in_repo": "afterburner/0.1.0/model.mpk"
    });
    uploaded_files[1] = serde_json::json!({
        "local_path": "<artifact>/manifest.toml",
        "path_in_repo": "afterburner/0.1.0/manifest.toml"
    });
    uploaded_files[2] = serde_json::json!({
        "local_path": "<request>",
        "path_in_repo": "afterburner/0.1.0/artifact_upload_request.json"
    });

    let expected_text =
        fs::read_to_string(fixture_path("huggingface_publish_receipt.example.json"))
            .expect("read receipt fixture");
    let expected: Value = serde_json::from_str(&expected_text).expect("parse fixture");
    assert_eq!(receipt, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("\"event\":\"huggingface_publish_receipt_written\""),
        "publish command must emit a receipt event"
    );
}

#[test]
fn hf_publish_rejects_non_huggingface_request_provider() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let request_path = tmp.path().join("artifact_upload_request.json");
    fs::write(
        &request_path,
        r#"{"schema_version":"1","operation":"upload","provider":"generic","destination":"uploads/afterburner","traceparent":"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01","trace_id":"4bf92f3577b34da6a3ce929d0e0e4736","artifact_version":"0.1.0","artifact_manifest":"artifacts/inference/0.1.0/manifest.toml","artifact_file":"artifacts/inference/0.1.0/model.mpk","artifact_directory":"artifacts/inference/0.1.0","artifact_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","provenance_source":"artifacts/train/train_done.jsonl","rollout_owner":"ml-release","approved_by":"ops-review","approval_ticket":"CHG-4242","approved_at_unix_ms":1735689600000}"#,
    )
    .expect("write request");

    let fake_hf_path = tmp.path().join("hf");
    let hf_log = tmp.path().join("hf.log");
    write_fake_hf(fake_hf_path.as_path(), hf_log.as_path());

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("hf-publish")
        .arg("--request")
        .arg(&request_path)
        .arg("--hf-bin")
        .arg(&fake_hf_path);
    cmd.assert().failure().code(2);
}

#[test]
fn hf_publish_uploads_optimized_package_support_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let request_path = tmp.path().join("artifact_upload_request.json");
    fs::write(
        &request_path,
        format!(
            r#"{{
  "schema_version":"1",
  "operation":"upload",
  "provider":"huggingface",
  "destination":"afterburner/model",
  "traceparent":"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
  "trace_id":"4bf92f3577b34da6a3ce929d0e0e4736",
  "artifact_version":"0.1.0",
  "artifact_manifest":"{0}",
  "artifact_file":"{1}",
  "artifact_directory":"{2}",
  "artifact_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "artifact_support_files":[
    {{"role":"optimization_profile","local_path":"{3}","path_in_artifact_directory":"model_optimization_profile.json"}},
    {{"role":"package_contract","local_path":"{4}","path_in_artifact_directory":"optimized_model_package_contract.json"}}
  ],
  "provenance_source":"artifacts/train/train_done.jsonl",
  "rollout_owner":"ml-release",
  "approved_by":"ops-review",
  "approval_ticket":"CHG-4242",
  "approved_at_unix_ms":1735689600000
}}"#,
            tmp.path().join("manifest.toml").display(),
            tmp.path().join("model.optimized.mpk").display(),
            tmp.path().display(),
            tmp.path().join("model_optimization_profile.json").display(),
            tmp.path().join("optimized_model_package_contract.json").display(),
        ),
    )
    .expect("write request");
    fs::write(tmp.path().join("manifest.toml"), "").expect("write manifest");
    fs::write(tmp.path().join("model.optimized.mpk"), "").expect("write artifact");
    fs::write(tmp.path().join("model_optimization_profile.json"), "").expect("write profile");
    fs::write(tmp.path().join("optimized_model_package_contract.json"), "")
        .expect("write package contract");

    let fake_hf_path = tmp.path().join("hf");
    let hf_log = tmp.path().join("hf.log");
    write_fake_hf(fake_hf_path.as_path(), hf_log.as_path());

    let receipt_path = tmp.path().join("huggingface_publish_receipt.json");
    let mut publish_cmd = cargo_bin_cmd!("afterburner");
    publish_cmd
        .arg("deploy")
        .arg("hf-publish")
        .arg("--request")
        .arg(&request_path)
        .arg("--hf-bin")
        .arg(&fake_hf_path)
        .arg("--revision")
        .arg("main")
        .arg("--out")
        .arg(&receipt_path);
    publish_cmd.assert().success();

    let log_text = fs::read_to_string(&hf_log).expect("read hf log");
    let log_lines = log_text.lines().collect::<Vec<_>>();
    assert_eq!(
        log_lines.len(),
        5,
        "must upload artifact, manifest, request, and two support files"
    );
    assert!(log_lines[3].contains("afterburner/0.1.0/model_optimization_profile.json"));
    assert!(log_lines[4].contains("afterburner/0.1.0/optimized_model_package_contract.json"));
}
