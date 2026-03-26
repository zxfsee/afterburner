use std::fs;
use std::path::PathBuf;

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
