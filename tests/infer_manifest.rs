use std::fs;

use afterburner::manifest::{MANIFEST_FILENAME, compute_sha256_hex};

#[path = "fixture_support.rs"]
mod fixture_support;

#[test]
fn infer_fails_fast_on_manifest_mismatch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    let weights = dir.join("model.mpk");
    fs::write(&weights, "").expect("write weights");
    let checksum = compute_sha256_hex(&weights).expect("checksum");
    let manifest_fixture = fixture_support::load_manifest_fixture();
    let bad_checksum = format!("{checksum}00");
    let manifest = manifest_fixture.replace(
        &format!("artifact_sha256 = \"{checksum}\""),
        &format!("artifact_sha256 = \"{bad_checksum}\""),
    );
    fs::write(dir.join(MANIFEST_FILENAME), manifest).expect("write manifest");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    cmd.arg("infer").arg(&weights);
    let assert = cmd.assert().failure().code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);

    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "stderr should have exactly one line");
    assert!(
        lines[0].contains(r#""event":"infer_error""#),
        "stderr should be structured JSON error"
    );
    assert!(
        lines[0].contains(r#""kind":"manifest_mismatch""#),
        "stderr should describe manifest mismatch"
    );
    assert!(
        lines[0].contains(r#""field":"artifact_sha256""#),
        "stderr should include checksum field"
    );
}

#[test]
fn infer_fails_fast_on_unsupported_quantized_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    let weights = dir.join("model.mpk");
    fs::write(&weights, "").expect("write weights");
    let checksum = compute_sha256_hex(&weights).expect("checksum");
    let manifest = fixture_support::load_manifest_fixture()
        .replace(
            r#"artifact_sha256 = "b3b57b1fea16e57389145b129a3330998bf3c3bc1e412c46779b3e1827fdd55b""#,
            &format!(r#"artifact_sha256 = "{checksum}""#),
        )
        .replace(r#"quantization = "none""#, r#"quantization = "int8""#);
    fs::write(dir.join(MANIFEST_FILENAME), manifest).expect("write manifest");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    cmd.arg("infer").arg(&weights);
    let assert = cmd.assert().failure().code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);

    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "stderr should have exactly one line");
    assert!(
        lines[0].contains(r#""event":"infer_error""#),
        "stderr should be structured JSON error"
    );
    assert!(
        lines[0].contains(r#""kind":"manifest_invalid_field""#),
        "stderr should describe invalid precision metadata"
    );
    assert!(
        lines[0].contains(r#""field":"precision.quantization""#),
        "stderr should include precision.quantization field"
    );
}
