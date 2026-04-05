use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
#[path = "../src/cmd_infer.rs"]
mod cmd_infer;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn native_metal_backend_fit_is_documented_and_registered() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-040"),
        "architecture decisions index must link ADR-040"
    );

    let adr = repo_file("docs/adr/040-native-metal-backend-fit.md");
    for needle in [
        "BACKEND=metal",
        "burn::backend::wgpu::Metal",
        "compiler feature matrix",
        "framework adapter registry",
        "`wgpu` as the default",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-040 must mention `{needle}` as part of the Metal fit decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("BACKEND=cpu|wgpu|metal"),
        "reference index must document the explicit Metal backend option"
    );

    let cargo_toml = repo_file("Cargo.toml");
    assert!(
        cargo_toml.contains("\"metal\""),
        "Cargo.toml must enable Burn's metal feature and register the metal backend"
    );

    let registry = repo_file("fixtures/framework_adapter_registry.schema.json");
    assert!(
        registry.contains("\"adapter\": \"metal\""),
        "framework adapter registry fixture must declare the metal adapter"
    );

    let supported = cmd_infer::runtime_supported_backends();
    assert!(
        supported.contains(&"metal"),
        "runtime_supported_backends must include metal"
    );
}
