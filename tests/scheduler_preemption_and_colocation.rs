use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn scheduler_preemption_and_colocation_stance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-042"),
        "architecture decisions index must link ADR-042"
    );
    for needle in [
        "cooperative checkpoint/resume only",
        "transparent GPU",
        "GPU colocation",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the scheduler stance"
        );
    }

    let adr = repo_file("docs/adr/042-scheduler-preemption-and-colocation.md");
    for needle in [
        "queueing plus priority",
        "cooperative checkpoint/resume",
        "transparent GPU",
        "UVM/MPS",
        "GPU colocation",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-042 must mention `{needle}` as part of the scheduler policy decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("supported preemption is cooperative checkpoint/resume"),
        "reference index must document the scheduler preemption stance"
    );
}
