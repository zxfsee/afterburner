use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn cubecl_fit_decision_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-012"),
        "architecture decisions index must link ADR-012"
    );
    assert!(
        architecture.contains("CubeCL"),
        "architecture must mention the CubeCL fit decision"
    );
    assert!(
        architecture.contains("CubeK"),
        "architecture must mention the CubeK fit decision"
    );

    let adr = repo_file("docs/adr/012-cubecl-fit.md");
    assert!(
        adr.contains("CubeCL"),
        "ADR-012 must describe the CubeCL decision"
    );
    assert!(
        adr.contains("CubeK"),
        "ADR-012 must describe the CubeK decision"
    );
    assert!(
        adr.contains("transitive"),
        "ADR-012 must make the transitive-dependency stance explicit"
    );

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("CubeCL"),
        "reference index must mention the CubeCL fit stance"
    );
    assert!(
        readme.contains("CubeK"),
        "reference index must mention the CubeK fit stance"
    );
}
