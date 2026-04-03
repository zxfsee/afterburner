use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn tracked_doc_surface_reads(text: &str) -> bool {
    text.contains("repo_file(\"README.md\")")
        || text.contains("repo_file(\"docs/reference.md\")")
        || text.contains("repo_file(\"docs/workflows.md\")")
}

fn brittle_doc_policing(text: &str) -> bool {
    text.contains("workflow reference must mention")
        || text.contains("reference index must mention")
        || text.contains("README must mention")
        || text.contains("README frontpage must")
        || text.contains("README must keep")
        || text.contains("README must point")
}

fn grouped_doc_surface_allowlist(file_name: &str) -> bool {
    matches!(
        file_name,
        "readme_frontpage.rs"
            | "readme_reference_split.rs"
            | "workflow_reference.rs"
            | "canonical_just_surface.rs"
            | "architecture_reference_ownership.rs"
            | "docs_ownership_separation.rs"
            | "deployment_verification_workflow_surface.rs"
            | "scheduler_heartbeat_workflow_surface.rs"
            | "doc_test_admission.rs"
    )
}

fn load_snapshot(path: &str) -> BTreeSet<String> {
    repo_file(path)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn doc_test_admission_stays_on_the_known_legacy_surface() {
    let doc_reader_allowlist = load_snapshot("tests/fixtures/doc_test_admission_allowlist.txt");
    let string_policing_allowlist =
        load_snapshot("tests/fixtures/doc_test_string_policing_allowlist.txt");

    let mut unexpected_doc_readers = Vec::new();
    let mut unexpected_string_policing = Vec::new();

    for entry in fs::read_dir(repo_root().join("tests")).expect("read tests dir") {
        let entry = entry.expect("read test entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("test filename");
        let rel_path = format!("tests/{file_name}");
        let text =
            fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", rel_path));

        if tracked_doc_surface_reads(&text)
            && !grouped_doc_surface_allowlist(file_name)
            && !doc_reader_allowlist.contains(&rel_path)
        {
            unexpected_doc_readers.push(rel_path.clone());
        }
        if brittle_doc_policing(&text)
            && !grouped_doc_surface_allowlist(file_name)
            && !string_policing_allowlist.contains(&rel_path)
        {
            unexpected_string_policing.push(rel_path);
        }
    }

    assert!(
        unexpected_doc_readers.is_empty(),
        "new doc-reading tests must join the grouped-surface allowlist or the transitional snapshot: {:?}",
        unexpected_doc_readers
    );
    assert!(
        unexpected_string_policing.is_empty(),
        "new docs-string policing tests must join the grouped-surface allowlist or the transitional snapshot: {:?}",
        unexpected_string_policing
    );
}

#[test]
fn doc_test_snapshots_only_keep_still_relevant_legacy_entries() {
    let doc_reader_allowlist = load_snapshot("tests/fixtures/doc_test_admission_allowlist.txt");
    let string_policing_allowlist =
        load_snapshot("tests/fixtures/doc_test_string_policing_allowlist.txt");

    let stale_doc_readers = doc_reader_allowlist
        .iter()
        .filter(|rel_path| {
            let text = repo_file(rel_path);
            !tracked_doc_surface_reads(&text)
        })
        .cloned()
        .collect::<Vec<_>>();
    let stale_string_policing = string_policing_allowlist
        .iter()
        .filter(|rel_path| {
            let text = repo_file(rel_path);
            !brittle_doc_policing(&text)
        })
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        stale_doc_readers.is_empty(),
        "doc-reader legacy snapshot must not keep stale entries: {:?}",
        stale_doc_readers
    );
    assert!(
        stale_string_policing.is_empty(),
        "docs-string legacy snapshot must not keep stale entries: {:?}",
        stale_string_policing
    );
}
