use std::fs;
use std::path::{Path, PathBuf};

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).expect("read dir");
    for entry in entries {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn train_module_is_split_into_responsibility_files() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    assert!(
        !repo.join("src/train.rs").exists(),
        "legacy src/train.rs should be removed once train responsibilities are split"
    );

    for path in [
        "src/train/mod.rs",
        "src/train/runtime.rs",
        "src/train/artifacts.rs",
        "src/train/distributed_metadata.rs",
        "src/train/observability.rs",
        "src/train/contracts.rs",
    ] {
        assert!(
            repo.join(path).is_file(),
            "missing split train module file `{path}`"
        );
    }

    let architecture = fs::read_to_string(repo.join("ARCHITECTURE.md"))
        .unwrap_or_else(|err| panic!("read ARCHITECTURE.md: {err}"));
    assert!(
        architecture.contains("train module should stay split by responsibility"),
        "architecture must mention the split train-module responsibility rule"
    );
}

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

#[test]
fn core_crate_does_not_depend_on_transport_only_crates() {
    let core_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("crates")
        .join("afterburner-core");
    let src_root = core_root.join("src");
    let core_manifest = fs::read_to_string(core_root.join("Cargo.toml")).expect("read core Cargo");
    let mut rust_files = Vec::new();
    collect_rust_files(&src_root, &mut rust_files);

    assert!(
        !core_manifest.contains("tiny_http"),
        "afterburner-core Cargo.toml must not depend on `tiny_http`"
    );
    assert!(
        !core_manifest.contains("ctrlc"),
        "afterburner-core Cargo.toml must not depend on `ctrlc`"
    );
    assert!(
        !core_manifest.contains("ratatui"),
        "afterburner-core Cargo.toml must not depend on `ratatui`"
    );

    for path in rust_files {
        let contents = fs::read_to_string(&path).expect("read rust file");
        assert!(
            !contents.contains("tiny_http"),
            "afterburner-core source must not depend on transport-only crate `tiny_http`: {}",
            path.display()
        );
        assert!(
            !contents.contains("ctrlc::"),
            "afterburner-core source must not depend on transport-only crate `ctrlc`: {}",
            path.display()
        );
        assert!(
            !contents.contains("ratatui"),
            "afterburner-core source must not depend on transport-only crate `ratatui`: {}",
            path.display()
        );
    }
}

#[test]
fn workspace_declares_core_member_and_adapter_dependency() {
    let root_manifest =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read root Cargo");

    assert!(
        root_manifest.contains("[workspace]"),
        "root Cargo.toml must declare a workspace"
    );
    assert!(
        root_manifest.contains("crates/afterburner-core"),
        "workspace must include the core crate member"
    );
    assert!(
        root_manifest.contains("afterburner-core = { path = \"crates/afterburner-core\" }"),
        "adapter package must depend on afterburner-core by path"
    );
}
