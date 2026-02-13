use std::path::PathBuf;
use std::process::Command;
use std::thread;

use afterburner::manifest::compute_sha256_hex;

fn artifact_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("inference")
        .join("0.1.0")
        .join("model.mpk")
}

fn run_infer_once() {
    let status = Command::new(assert_cmd::cargo::cargo_bin!("afterburner"))
        .arg("infer")
        .arg(artifact_path())
        .env("BACKEND", "cpu")
        .status()
        .expect("spawn infer process");
    assert!(status.success(), "infer process must succeed");
}

#[test]
fn concurrent_readers_do_not_mutate_pinned_artifact() {
    let artifact = artifact_path();
    let before = compute_sha256_hex(&artifact).expect("compute pre-run checksum");

    let worker_a = thread::spawn(run_infer_once);
    let worker_b = thread::spawn(run_infer_once);
    worker_a.join().expect("join worker a");
    worker_b.join().expect("join worker b");

    let after = compute_sha256_hex(&artifact).expect("compute post-run checksum");
    assert_eq!(before, after, "artifact checksum must remain unchanged");
}
