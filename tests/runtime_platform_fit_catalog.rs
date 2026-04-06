#[path = "support/repo.rs"]
mod repo_test_support;

use afterburner::RUNTIME_SUPPORTED_BACKENDS;
use repo_test_support::repo_file;

#[test]
fn runtime_backend_strategy_is_documented_and_registered() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-012"),
        "architecture decisions index must link ADR-012"
    );

    let adr = repo_file("docs/adr/012-runtime-backend-strategy.md");
    for needle in [
        "Burn-aligned",
        "BACKEND=cpu|wgpu|metal",
        "burn::backend::wgpu::Metal",
        "`wgpu` as the default",
        "CubeCL",
        "CubeK",
        "`cutile-rs`",
        "compiler feature matrix",
        "framework adapter registry",
        "NVIDIA/CUDA",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-012 must mention `{needle}` as part of the runtime backend strategy"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("BACKEND=cpu|wgpu|metal"),
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

    let supported = RUNTIME_SUPPORTED_BACKENDS;
    assert!(
        supported.contains(&"metal"),
        "runtime_supported_backends must include metal"
    );
    assert!(
        reference.contains("CubeCL"),
        "reference index must mention the CubeCL fit stance"
    );
    assert!(
        reference.contains("CubeK"),
        "reference index must mention the CubeK fit stance"
    );
    assert!(
        reference.contains("later NVIDIA-specific backend-extension candidate"),
        "reference index must mention the cutile-rs fit stance"
    );
}

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

    let reference = repo_file("docs/reference.md");
    for needle in ["Parquet", "DataFusion", "Ballista"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the current data-infra fit stance `{needle}`"
        );
    }
}

#[test]
fn process_compose_local_substrate_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-048"),
        "architecture decisions index must link ADR-048"
    );
    for needle in [
        "`process-compose-flake`",
        "local substrate",
        "`systemd`",
        "Kubernetes",
        "scheduler policy",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the local substrate fit"
        );
    }

    let adr = repo_file("docs/adr/048-process-compose-local-substrate-fit.md");
    for needle in [
        "`process-compose-flake`",
        "MacBook-local",
        "`systemd`",
        "Kubernetes",
        "local process substrate",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-048 must mention `{needle}` as part of the fit decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("process-compose-flake"),
        "reference index must document the local process-compose fit"
    );
}
