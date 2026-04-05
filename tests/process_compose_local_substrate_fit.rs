use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn process_compose_local_substrate_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-048"),
        "architecture decisions index must link ADR-048"
    );
    for needle in [
        "`process-compose-flake`",
        "local substrate",
        "`systemd`",
        "Kubernetes",
        "scheduler policy",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the local substrate fit"
        );
    }

    let adr = repo_file("docs/adr/048-process-compose-local-substrate-fit.md");
    for needle in [
        "`process-compose-flake`",
        "MacBook-local",
        "`systemd`",
        "Kubernetes",
        "local process substrate",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-048 must mention `{needle}` as part of the fit decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("process-compose-flake"),
        "reference index must document the local process-compose fit"
    );
}
