use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

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
