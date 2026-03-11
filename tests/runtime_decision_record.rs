use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn async_runtime_decision_record_is_linked_and_explicit() {
    let architecture =
        fs::read_to_string(repo_root().join("ARCHITECTURE.md")).expect("read architecture");
    let adr_path = repo_root().join("docs/adr/007-async-runtime-decision.md");
    let adr = fs::read_to_string(&adr_path).expect("read async runtime ADR");
    let architecture_lower = architecture.to_ascii_lowercase();

    assert!(
        architecture.contains("ADR-007"),
        "architecture decisions index must link the async runtime ADR"
    );
    assert!(
        architecture_lower.contains("no async runtime is adopted"),
        "architecture must state the current runtime decision explicitly"
    );
    assert!(
        adr.contains("Tokio"),
        "ADR must discuss the Tokio decision explicitly"
    );
    assert!(
        adr.contains("tiny_http"),
        "ADR must record the current HTTP adapter baseline"
    );
    assert!(
        adr.contains("profil"),
        "ADR must record what evidence would justify revisiting the decision"
    );
}
