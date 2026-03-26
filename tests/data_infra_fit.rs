use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn arrow_datafusion_ballista_parquet_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-011"),
        "architecture decisions index must link ADR-011"
    );
    assert!(
        architecture.contains("Parquet"),
        "architecture must mention the Parquet fit decision"
    );
    assert!(
        architecture.contains("DataFusion"),
        "architecture must mention the DataFusion fit decision"
    );
    assert!(
        architecture.contains("Ballista"),
        "architecture must mention the Ballista fit decision"
    );

    let adr = repo_file("docs/adr/011-data-infra-fit.md");
    assert!(
        adr.contains("Parquet"),
        "ADR-011 must describe the Parquet decision"
    );
    assert!(
        adr.contains("DataFusion"),
        "ADR-011 must describe the DataFusion decision"
    );
    assert!(
        adr.contains("Ballista"),
        "ADR-011 must describe the Ballista decision"
    );
    assert!(
        adr.contains("no dependency"),
        "ADR-011 must make the current no-dependency stance explicit"
    );

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("Parquet"),
        "reference index must mention the current Parquet fit stance"
    );
    assert!(
        readme.contains("DataFusion"),
        "reference index must mention the current DataFusion fit stance"
    );
}
