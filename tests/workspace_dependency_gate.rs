use std::fs;
use std::path::PathBuf;

#[test]
fn workspace_declares_core_crate_and_adapter_dependency_gate() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_manifest =
        fs::read_to_string(repo_root.join("Cargo.toml")).expect("read root Cargo.toml");
    assert!(
        root_manifest.contains("[workspace]"),
        "root manifest must declare a cargo workspace"
    );
    assert!(
        root_manifest.contains("afterburner-core = { path = \"crates/afterburner-core\" }"),
        "root manifest must depend on the core crate via path dependency"
    );

    let core_manifest_path = repo_root
        .join("crates")
        .join("afterburner-core")
        .join("Cargo.toml");
    let core_manifest =
        fs::read_to_string(&core_manifest_path).expect("read crates/afterburner-core/Cargo.toml");
    assert!(
        core_manifest.contains("name = \"afterburner-core\""),
        "core crate manifest must declare the afterburner-core package"
    );
    for forbidden in ["tiny_http", "ctrlc", "ratatui"] {
        assert!(
            !core_manifest.contains(forbidden),
            "core crate manifest must not depend on adapter-only crate `{forbidden}`"
        );
    }
}
