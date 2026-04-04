use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn profile_infer_recipe(justfile: &str) -> &str {
    justfile
        .split("profile-infer:")
        .nth(1)
        .and_then(|rest| rest.split("\n\n# open the terminal dashboard").next())
        .expect("profile-infer recipe block")
}

#[test]
fn profiling_recipe_delegates_current_pointer_and_backend_resolution_to_native_command() {
    let justfile = repo_file("justfile");
    let recipe = profile_infer_recipe(&justfile);
    assert!(
        recipe.contains("cargo run --locked --bin afterburner -- profile infer"),
        "profile-infer must delegate to the native profile infer command"
    );
    assert!(
        !recipe.contains("open artifacts/inference/current"),
        "profile-infer must not keep current-pointer resolution in recipe-local shell logic"
    );
    assert!(
        !recipe.contains("if (\"BACKEND\" in $env)"),
        "profile-infer must not keep backend derivation in recipe-local shell logic"
    );
    assert!(
        !recipe.contains("afterburner_profile_summary"),
        "profile-infer must not shell out to the summary helper directly once the native command owns the workflow"
    );
    assert!(
        !recipe.contains("profile-environment-snapshot"),
        "profile-infer must not re-enter the environment snapshot recipe once the native command owns the workflow"
    );
    assert!(
        !recipe.contains("profile-infer-artifact := \"artifacts/inference/0.1.0/model.mpk\""),
        "profile-infer must not pin a specific artifact version in justfile"
    );
    assert!(
        !recipe.contains("profile-infer-backend := \"wgpu\""),
        "profile-infer must not pin a fixed backend variable in justfile"
    );

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("afterburner profile infer"),
        "workflow reference should document the native profile infer command behind the stable just entrypoint"
    );
    assert!(
        readme.contains("current pointer") && readme.contains("BACKEND"),
        "workflow reference should still explain the current pointer and backend contracts"
    );
}
