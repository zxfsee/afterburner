use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn json_over_ron_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-060"),
        "architecture decisions index must link ADR-060"
    );
    assert!(
        architecture.contains("Structured artifact and event contracts stay on JSON"),
        "architecture must make the keep-JSON stance explicit"
    );
    assert!(
        architecture.contains("manifest remains TOML"),
        "architecture must keep the manifest format distinction explicit"
    );

    let adr = repo_file("docs/adr/060-json-over-ron-fit.md");
    for needle in [
        "JSON",
        "RON",
        "machine-readable artifact and event contracts stay on JSON",
        "inference manifest stays on TOML",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-060 must mention `{needle}` as part of the format-fit decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("Structured artifacts and events stay on JSON"),
        "README must mention the keep-JSON artifact/event stance"
    );
    assert!(
        readme.contains("manifest stays on TOML"),
        "README must mention the manifest format distinction"
    );
}
