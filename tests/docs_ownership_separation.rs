#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn docs_ownership_separation_is_explicit() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("[Workflow reference](./docs/workflows.md)"),
        "README must point to docs/workflows.md"
    );
    assert!(
        readme.contains("[Reference index](./docs/reference.md)"),
        "README must point to docs/reference.md"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("Operational workflow details live here"),
        "workflow doc must state its ownership role"
    );

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("Contract and capability detail lives here"),
        "reference doc must state its ownership role"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("system shape, stable boundaries,"),
        "architecture must stay scoped to system shape and invariants"
    );
}
