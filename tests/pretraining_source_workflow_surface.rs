use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn pretraining_source_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "pretraining-source-approval source source_revision approval_status approved_by approval_ticket approved_at_unix_ms:",
        "pretraining-source-provenance action +args:",
        "workflow-surface-check-pretraining-source:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "pretraining-source-approval-receipt source source_revision approval_status approved_by approval_ticket approved_at_unix_ms:",
        "pretraining-source-provenance-receipt approval_receipt registry_entry_path upstream_locator reviewed_metadata_sha256:",
        "pretraining-source-provenance-evidence-bundle provenance_receipt:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep flat source recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Pretraining source stays grouped under two workflow entrypoints:",
        "`just pretraining-source-approval` writes `pretraining_source_approval_receipt.json`.",
        "`just pretraining-source-provenance <receipt|bundle>` covers `pretraining_source_provenance_receipt.json` and `pretraining_source_provenance_evidence_bundle.json`.",
        "`just workflow-surface-check-pretraining-source` keeps the grouped CLI and recipe surface checked.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped pretraining source surface `{needle}`"
        );
    }

    for forbidden in [
        "just pretraining-source-approval-receipt",
        "just pretraining-source-provenance-receipt",
        "just pretraining-source-provenance-evidence-bundle",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating source variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Pretraining source family groups:",
        "`pretraining_source_approval_receipt.json`",
        "`pretraining_source_provenance_receipt.json`",
        "`pretraining_source_provenance_evidence_bundle.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped entrypoint map and `workflow-surface-check-pretraining-source` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped pretraining source surface `{needle}`"
        );
    }
}
