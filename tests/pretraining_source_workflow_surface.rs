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
        "pretraining-source-provenance-receipt approval_receipt registry_entry_path upstream_locator reviewed_metadata_sha256:",
        "pretraining-source-provenance-evidence-bundle provenance_receipt:",
        "workflow-surface-check-pretraining-source:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "pretraining-source-approval-receipt source source_revision approval_status approved_by approval_ticket approved_at_unix_ms:",
        "pretraining-source-provenance action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep generic source bucket `{forbidden}` after collapse"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Pretraining source stays grouped under these operator flows:",
        "Approval flow: `just pretraining-source-approval`.",
        "Provenance flow: `just pretraining-source-provenance-receipt`, `just pretraining-source-provenance-evidence-bundle`.",
        "`just workflow-surface-check-pretraining-source` guards the grouped source workflow map and recipe surface.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped pretraining source surface `{needle}`"
        );
    }

    for forbidden in [
        "just pretraining-source-approval-receipt",
        "just pretraining-source-provenance <receipt|bundle>",
        "- `just pretraining-source-approval` writes `pretraining_source_approval_receipt.json`.",
        "- `just pretraining-source-provenance-receipt` writes `pretraining_source_provenance_receipt.json`.",
        "- `just pretraining-source-provenance-evidence-bundle` writes `pretraining_source_provenance_evidence_bundle.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating source variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Pretraining source artifact groups:",
        "`pretraining_source_approval_receipt.json`",
        "`pretraining_source_provenance_receipt.json`",
        "`pretraining_source_provenance_evidence_bundle.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-pretraining-source` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped pretraining source surface `{needle}`"
        );
    }
}
