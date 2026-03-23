use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn macbook_text_pretraining_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-047"),
        "architecture decisions index must link ADR-047"
    );
    for needle in [
        "decoder-only language model",
        "single-node, single-device execution",
        "`50M` to `300M` parameters",
        "`1024` token context length",
        "`50M` to `200M` token budgets",
        "`AdamW`",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the MacBook text-pretraining fit"
        );
    }

    let adr = repo_file("docs/adr/047-macbook-text-pretraining-fit.md");
    for needle in [
        "decoder-only language model",
        "`50M` to `300M`",
        "`1024`",
        "`50M` to `200M`",
        "`AdamW`",
        "checkpoint cadence",
        "tiny-overfit",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-047 must mention `{needle}` as part of the workload envelope"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("decoder-only language model"),
        "README must mention the bounded MacBook text-pretraining target"
    );
}
