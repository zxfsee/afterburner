use std::fs;
use std::path::PathBuf;

#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn architecture_reference_ownership_is_explicit() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("Reference-owned stance catalog"),
        "architecture must section the reference-owned stance catalog explicitly"
    );
    assert!(
        architecture.contains("docs/reference.md"),
        "architecture must point capability and fit ownership at docs/reference.md"
    );

    let reference = repo_file("docs/reference.md");
    for heading in [
        "## Training and inference contracts",
        "## Workflow and artifact families",
        "## Capability and stance summary",
    ] {
        assert!(
            reference.contains(heading),
            "reference index must contain `{heading}`"
        );
    }
}

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

#[test]
fn readme_frontpage_is_sharp_and_scannable() {
    let readme = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .unwrap_or_else(|err| panic!("read README.md: {err}"));

    let first_block = readme.lines().take(80).collect::<Vec<_>>().join("\n");
    for heading in [
        "## Quick start",
        "## At a glance",
        "## System shape",
        "## Read next",
    ] {
        assert!(
            first_block.contains(heading),
            "README frontpage must include `{heading}` near the top"
        );
    }

    assert!(
        first_block.contains("The canonical workflow surface is `just`"),
        "README frontpage must state the canonical workflow surface near the top"
    );
    assert!(
        first_block.contains("[Architecture](./ARCHITECTURE.md)")
            && first_block.contains("[ADRs](./docs/adr/)")
            && first_block.contains("[Changelog](./CHANGELOG.md)"),
        "README frontpage must point readers to deeper repo docs near the top"
    );
}
