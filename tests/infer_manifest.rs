use std::fs;

use afterburner::manifest::MANIFEST_FILENAME;

#[test]
fn infer_fails_fast_on_manifest_mismatch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    let weights = dir.join("model.mpk");
    fs::write(&weights, "").expect("write weights");

    let manifest = r#"
artifact = "model.mpk"

[model]
architecture_id = "afterburner.mnist.residual_v0"
architecture_version = 1

[input]
shape = [1, 28, 28]
dtype = "f32"

[normalization]
dataset = "mnist"
mean = 0.1307
std = 0.3081
notes = "((x / 255.0) - 0.1307) / 0.3081"
"#;
    fs::write(dir.join(MANIFEST_FILENAME), manifest).expect("write manifest");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("infer");
    cmd.arg(&weights);
    let assert = cmd.assert().failure();
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
}
