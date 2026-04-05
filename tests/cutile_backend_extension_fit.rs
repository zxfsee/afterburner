#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn cutile_backend_extension_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-057"),
        "architecture decisions index must link ADR-057"
    );

    let adr = repo_file("docs/adr/057-cutile-backend-extension-fit.md");
    for needle in [
        "`cutile-rs`",
        "CubeCL/CubeK",
        "NVIDIA/CUDA-oriented",
        "not the preferred current path",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-057 must mention `{needle}` as part of the fit decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("later NVIDIA-specific backend-extension candidate"),
        "reference index must mention the cutile-rs fit stance"
    );
}
