use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn async_runtime_decision_record_is_linked_and_explicit() {
    let architecture = repo_file("ARCHITECTURE.md");
    let adr = repo_file("docs/adr/007-async-runtime-decision.md");
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
    for needle in ["concurrency", "deployment", "trace"] {
        assert!(
            adr.to_ascii_lowercase().contains(needle),
            "ADR must mention `{needle}` as part of the Tokio fit criteria"
        );
    }
    assert!(
        adr.contains("profil"),
        "ADR must record what evidence would justify revisiting the decision"
    );
}

#[test]
fn async_runtime_fit_stays_deferred_across_reference_surfaces() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("synchronous `tiny_http`"),
        "README must keep the HTTP adapter baseline explicit"
    );
    assert!(
        readme.contains("Tokio"),
        "README must mention the deferred Tokio fit trigger"
    );

    let cargo_toml = repo_file("Cargo.toml");
    assert!(
        !cargo_toml.contains("tokio ="),
        "Cargo.toml must not add Tokio before the adapter runtime gate is intentionally revisited"
    );
}
