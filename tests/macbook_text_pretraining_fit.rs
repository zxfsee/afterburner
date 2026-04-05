use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn macbook_text_pretraining_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-047"),
        "architecture decisions index must link ADR-047"
    );
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

    let readme = repo_file("docs/reference.md");
    for needle in [
        "decoder-only language model",
        "single-node, single-device execution",
        "`50M` to `300M` parameters",
        "`1024` token context length",
        "`50M` to `200M` token budgets",
        "`AdamW`",
    ] {
        assert!(
            readme.contains(needle),
            "reference index must mention the bounded MacBook text-pretraining detail `{needle}`"
        );
    }
}
