use std::fs;
use std::path::PathBuf;

use afterburner::manifest::ArtifactManifest;
use serde_json::Value;

mod support;

use support::fixture_path;

#[test]
fn profile_infer_command_writes_summary_and_environment_snapshot() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact_dir = tmp.path().join("artifact");
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&artifact_dir).expect("create artifact dir");
    fs::create_dir_all(&bin_dir).expect("create bin dir");

    let weights_path = artifact_dir.join("model.mpk");
    fs::write(&weights_path, b"fixture-model").expect("write weights");
    ArtifactManifest::for_current("model.mpk", "0.1.0", "fixture-checksum".to_string())
        .write_to_dir(&artifact_dir)
        .expect("write manifest");

    let flamegraph_fixture = fixture_path("profiling_flamegraph_fixture.svg");
    let cargo_script = bin_dir.join("cargo");
    fs::write(
        &cargo_script,
        format!(
            "#!/bin/sh\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-o\" ]; then\n    out=\"$2\"\n    shift 2\n    continue\n  fi\n  shift\n done\ncp \"{}\" \"$out\"\n",
            flamegraph_fixture.display()
        ),
    )
    .expect("write fake cargo");

    let profiler_script = bin_dir.join("cargo-flamegraph");
    fs::write(&profiler_script, "#!/bin/sh\necho cargo-flamegraph 9.9.9\n")
        .expect("write fake profiler");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&cargo_script, &profiler_script] {
            let mut permissions = fs::metadata(path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).expect("chmod");
        }
    }

    let flamegraph_out = tmp.path().join("infer_flamegraph.svg");
    let summary_out = tmp.path().join("infer_hotspot_summary.json");
    let snapshot_out = tmp.path().join("profiling_environment_snapshot.json");
    let path_env = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").expect("PATH")
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    cmd.arg("profile")
        .arg("infer")
        .arg("--artifact")
        .arg(&weights_path)
        .arg("--backend")
        .arg("cpu")
        .arg("--flamegraph-out")
        .arg(&flamegraph_out)
        .arg("--summary-out")
        .arg(&summary_out)
        .arg("--snapshot-out")
        .arg(&snapshot_out)
        .arg("--profiler-path")
        .arg(&profiler_script)
        .env("PATH", path_env)
        .assert()
        .success();

    let summary_text = fs::read_to_string(&summary_out).expect("read summary");
    let summary: Value = serde_json::from_str(&summary_text).expect("parse summary");
    assert_eq!(summary["profile_kind"], "infer");
    assert_eq!(summary["backend"], "cpu");
    assert_eq!(summary["artifact_version"], "0.1.0");
    assert_eq!(
        summary["weights_artifact"],
        weights_path.display().to_string()
    );
    assert!(
        summary["profile_command"]
            .as_str()
            .is_some_and(|value| value.contains("cargo flamegraph")
                && value.contains(&weights_path.display().to_string())),
        "profile command must record the infer flamegraph invocation"
    );

    let snapshot_text = fs::read_to_string(&snapshot_out).expect("read snapshot");
    let snapshot: Value = serde_json::from_str(&snapshot_text).expect("parse snapshot");
    assert_eq!(snapshot["profile_kind"], "infer");
    assert_eq!(snapshot["profiler"], "cargo-flamegraph");
    assert_eq!(
        snapshot["profiler_path"],
        profiler_script.display().to_string()
    );
    assert_eq!(snapshot["profiler_version"], "cargo-flamegraph 9.9.9");
}
