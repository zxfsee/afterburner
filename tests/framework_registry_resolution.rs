use std::fs;
use std::path::Path;
use std::path::PathBuf;

use afterburner::manifest::{MANIFEST_FILENAME, compute_sha256_hex};
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[path = "fixture_support.rs"]
mod fixture_support;

#[test]
fn infer_allows_each_supported_backend_artifact_version_pair_from_fixture_metadata() {
    let supported_pairs = load_supported_pairs_from_fixture();
    assert!(
        !supported_pairs.is_empty(),
        "fixture must declare at least one supported backend/artifact pair"
    );

    for (backend, artifact_version) in supported_pairs {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifact_dir = tmp.path().join("artifact");
        fs::create_dir_all(&artifact_dir).expect("create artifact dir");

        let weights_path = artifact_dir.join("model.mpk");
        fs::write(&weights_path, "").expect("write placeholder model");

        let manifest = manifest_with(&weights_path, artifact_version.as_str());
        fs::write(artifact_dir.join(MANIFEST_FILENAME), manifest).expect("write manifest");

        let mut cmd = cargo_bin_cmd!("afterburner");
        cmd.arg("infer")
            .arg("--artifact")
            .arg(&weights_path)
            .env("BACKEND", backend.as_str());

        cmd.assert().failure().code(2).stderr(
            predicate::str::contains(r#""event":"infer_error""#)
                .and(predicate::str::contains(r#""kind":"artifact_load_failed""#))
                .and(predicate::str::contains(r#""kind":"adapter_registry_unsupported""#).not()),
        );
    }
}

#[test]
fn infer_fails_fast_on_unsupported_backend_artifact_version_pair() {
    let supported_pairs = load_supported_pairs_from_fixture();
    let (backend, supported_version) = supported_pairs
        .first()
        .expect("fixture must declare at least one supported backend/artifact pair");
    let unsupported_version = if supported_version == "99.99.99" {
        "98.98.98".to_string()
    } else {
        "99.99.99".to_string()
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let weights_path = artifact_dir.join("model.mpk");
    fs::write(&weights_path, "").expect("write placeholder model");

    let manifest = manifest_with(&weights_path, unsupported_version.as_str());
    fs::write(artifact_dir.join(MANIFEST_FILENAME), manifest).expect("write manifest");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer")
        .arg("--artifact")
        .arg(&weights_path)
        .env("BACKEND", backend.as_str());

    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"infer_error""#)
            .and(predicate::str::contains(
                r#""kind":"adapter_registry_unsupported""#,
            ))
            .and(predicate::str::contains(format!(
                r#""backend":"{backend}""#
            )))
            .and(predicate::str::contains(format!(
                r#""artifact_version":"{unsupported_version}""#
            )))
            .and(predicate::str::contains(r#""kind":"artifact_load_failed""#).not()),
    );
}

fn load_supported_pairs_from_fixture() -> Vec<(String, String)> {
    let schema_text = fs::read_to_string(fixture_path("framework_adapter_registry.schema.json"))
        .expect("read framework adapter registry schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");
    let entries = schema
        .get("x-supported-adapter-version-pairs")
        .and_then(|v| v.as_array())
        .expect("schema.x-supported-adapter-version-pairs must be an array");

    entries
        .iter()
        .map(|entry| {
            let entry = entry
                .as_object()
                .expect("supported adapter/version pair entries must be objects");
            let backend = entry
                .get("adapter")
                .and_then(|v| v.as_str())
                .expect("supported pair adapter must be a string")
                .to_string();
            let version = entry
                .get("version")
                .and_then(|v| v.as_str())
                .expect("supported pair version must be a string")
                .to_string();
            (backend, version)
        })
        .collect()
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn manifest_with(weights_path: &Path, artifact_version: &str) -> String {
    let checksum = compute_sha256_hex(weights_path).expect("compute checksum");
    fixture_support::load_manifest_fixture()
        .replace(
            r#"artifact_version = "0.1.0""#,
            &format!(r#"artifact_version = "{artifact_version}""#),
        )
        .replace(
            r#"artifact_sha256 = "b3b57b1fea16e57389145b129a3330998bf3c3bc1e412c46779b3e1827fdd55b""#,
            &format!(r#"artifact_sha256 = "{checksum}""#),
        )
}
