use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod support;

use support::repo_file;

#[test]
fn canonical_just_surface_is_documented_and_discoverable() {
    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("workflows:"),
        "justfile must expose the workflows discovery recipe"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("The canonical workflow surface is `just`"),
        "README must declare just as the canonical workflow surface"
    );
    for needle in ["just --list", "just workflows"] {
        assert!(
            readme.contains(needle),
            "README must document `{needle}` as part of workflow discovery"
        );
    }

    let output = Command::new("just")
        .arg("--list")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap_or_else(|err| panic!("run just --list: {err}"));
    assert!(
        output.status.success(),
        "just --list must succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("just --list stdout utf8");
    for recipe in [
        "workflows",
        "train",
        "infer",
        "eval-gate",
        "train-text-smoke",
    ] {
        assert!(
            stdout.contains(recipe),
            "just --list output must include `{recipe}`"
        );
    }
}
