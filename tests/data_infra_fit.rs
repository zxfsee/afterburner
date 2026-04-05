#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn arrow_datafusion_ballista_parquet_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-011"),
        "architecture decisions index must link ADR-011"
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
    for needle in ["Parquet", "DataFusion", "Ballista"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the current data-infra fit stance `{needle}`"
        );
    }
}
