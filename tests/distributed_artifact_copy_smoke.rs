use std::path::PathBuf;
use std::process::Command;
use std::thread;

use afterburner::manifest::compute_sha256_hex;

#[path = "fixture_support.rs"]
mod fixture_support;

fn run_infer_once(artifact_path: PathBuf) {
    let status = Command::new(assert_cmd::cargo::cargo_bin!("afterburner"))
        .arg("infer")
        .arg(artifact_path)
        .env("BACKEND", "cpu")
        .status()
        .expect("spawn infer process");
    assert!(status.success(), "infer process must succeed");
}

#[test]
fn concurrent_readers_do_not_mutate_pinned_artifact() {
    let (_artifact_dir, artifact) = fixture_support::build_runtime_model_artifact();
    let before = compute_sha256_hex(&artifact).expect("compute pre-run checksum");

    let worker_a_artifact = artifact.clone();
    let worker_b_artifact = artifact.clone();
    let worker_a = thread::spawn(move || run_infer_once(worker_a_artifact));
    let worker_b = thread::spawn(move || run_infer_once(worker_b_artifact));
    worker_a.join().expect("join worker a");
    worker_b.join().expect("join worker b");

    let after = compute_sha256_hex(&artifact).expect("compute post-run checksum");
    assert_eq!(before, after, "artifact checksum must remain unchanged");
}
