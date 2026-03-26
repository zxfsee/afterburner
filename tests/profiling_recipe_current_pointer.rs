use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn profiling_recipe_resolves_current_pointer_and_active_backend() {
    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("open artifacts/inference/current"),
        "profile-infer must resolve the artifact version through artifacts/inference/current"
    );
    assert!(
        justfile.contains("if (\"BACKEND\" in $env)"),
        "profile-infer must derive the backend from the runtime env contract"
    );
    assert!(
        !justfile.contains("profile-infer-artifact := \"artifacts/inference/0.1.0/model.mpk\""),
        "profile-infer must not pin a specific artifact version in justfile"
    );
    assert!(
        !justfile.contains("profile-infer-backend := \"wgpu\""),
        "profile-infer must not pin a fixed backend variable in justfile"
    );

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("current pointer"),
        "workflow reference should explain that profiling resolves through the current pointer"
    );
    assert!(
        readme.contains("BACKEND"),
        "workflow reference should mention that profiling follows the active backend env contract"
    );
}
