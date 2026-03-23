use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn fineweb_edu_source_adoption_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-049"),
        "architecture decisions index must link ADR-049"
    );
    assert!(
        architecture.contains("fineweb-edu/slice"),
        "architecture must mention the FineWeb-Edu source family"
    );

    let adr = repo_file("docs/adr/049-fineweb-edu-source-adoption.md");
    for needle in [
        "`fineweb-edu/slice`",
        "`source_revision`",
        "bounded local slice snapshot",
        "tens of gigabytes",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-049 must mention `{needle}` as part of the source fit decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("fineweb-edu/slice"),
        "README must mention the FineWeb-Edu source fit"
    );
}
