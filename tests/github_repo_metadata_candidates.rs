use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn github_repo_metadata_candidates_are_documented() {
    let readme = repo_file("README.md");
    for needle in [
        "## GitHub metadata candidates",
        "Description candidate: Rust-first ML systems repo for training Burn models, exporting deployable inference artifacts, and exercising contract-first deployment and observability workflows.",
        "Topics candidate: `rust`, `machine-learning`, `mlops`, `burn`, `inference`, `training`, `nix`, `observability`, `deployment`, `artifacts`",
        "Website candidate: none for now",
    ] {
        assert!(
            readme.contains(needle),
            "README must contain `{needle}` for the GitHub metadata candidates doc"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("GitHub repo metadata source of truth currently starts in `README.md` under `GitHub metadata candidates`"),
        "reference index must point back to the README metadata candidates source of truth"
    );
}
