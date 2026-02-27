use std::fs;
use std::path::Path;

use afterburner::manifest::{MANIFEST_FILENAME, compute_sha256_hex};
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[path = "fixture_support.rs"]
mod fixture_support;

#[test]
fn infer_allows_supported_backend_artifact_version_pair() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let weights_path = artifact_dir.join("model.mpk");
    fs::write(&weights_path, "").expect("write placeholder model");

    let manifest = manifest_with(&weights_path, "0.1.0");
    fs::write(artifact_dir.join(MANIFEST_FILENAME), manifest).expect("write manifest");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer")
        .arg("--artifact")
        .arg(&weights_path)
        .env("BACKEND", "cpu");

    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"infer_error""#)
            .and(predicate::str::contains(r#""kind":"artifact_load_failed""#))
            .and(predicate::str::contains(r#""kind":"adapter_registry_unsupported""#).not()),
    );
}

#[test]
fn infer_fails_fast_on_unsupported_backend_artifact_version_pair() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");

    let weights_path = artifact_dir.join("model.mpk");
    fs::write(&weights_path, "").expect("write placeholder model");

    let manifest = manifest_with(&weights_path, "9.9.9");
    fs::write(artifact_dir.join(MANIFEST_FILENAME), manifest).expect("write manifest");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer")
        .arg("--artifact")
        .arg(&weights_path)
        .env("BACKEND", "cpu");

    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"infer_error""#)
            .and(predicate::str::contains(
                r#""kind":"adapter_registry_unsupported""#,
            ))
            .and(predicate::str::contains(r#""backend":"cpu""#))
            .and(predicate::str::contains(r#""artifact_version":"9.9.9""#))
            .and(predicate::str::contains(r#""kind":"artifact_load_failed""#).not()),
    );
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
