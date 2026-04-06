use std::collections::BTreeSet;
use std::fs;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn burn_dependency_refresh_stance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-033"),
        "architecture decisions index must link ADR-033"
    );
    let adr = repo_file("docs/adr/033-burn-dependency-refresh.md");
    for needle in ["0.20.1", "0.21.0-pre.1", "latest stable", ".mpk", ".bpk"] {
        assert!(
            adr.contains(needle),
            "ADR-033 must mention `{needle}` as part of the refresh decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in ["Burn dependency refresh", "Burn 0.20.1"] {
        assert!(
            reference.contains(needle),
            "reference index must mention the current Burn refresh stance `{needle}`"
        );
    }
}

#[test]
fn burn_stable_release_availability_gate_is_documented_against_the_current_pin() {
    let cargo_toml = repo_file("Cargo.toml");
    assert!(
        cargo_toml.contains("burn = { version = \"0.20.1\""),
        "Cargo.toml must keep the current Burn pin explicit for the availability gate"
    );
    assert!(
        cargo_toml.contains("[dependencies.burn-autodiff]\nversion = \"0.20.1\""),
        "Cargo.toml must keep the current burn-autodiff pin explicit for the availability gate"
    );

    let adr = repo_file("docs/adr/033-burn-dependency-refresh.md");
    for needle in [
        "checked latest stable Burn release: `0.20.1`",
        "newer stable Burn release beyond `0.20.1`: `no`",
        "keep the `.bpk` migration blocked until this checked state changes",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-033 must record `{needle}` as checked availability state"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("burn_stable_release_availability"),
        "reference index must mention the Burn stable release availability gate"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("cargo nextest run --locked --test burn_stable_release_availability"),
        "workflow reference must mention the Burn stable release availability gate command"
    );
}

#[test]
fn capnproto_wire_format_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-063"),
        "architecture decisions index must link ADR-063"
    );
    assert!(
        architecture.contains("capnproto-rust"),
        "architecture must mention the capnproto-rust fit decision"
    );
    assert!(
        architecture.contains("JSON/TOML"),
        "architecture must keep the current JSON/TOML stance explicit"
    );

    let adr = repo_file("docs/adr/063-capnproto-wire-format-fit.md");
    assert!(
        adr.contains("Do not adopt `capnproto-rust` now."),
        "ADR-063 must make the current no-adoption decision explicit"
    );
    assert!(
        adr.contains("Do not add a `capnproto-rust` dependency"),
        "ADR-063 must keep the current no-dependency stance explicit"
    );
    assert!(
        adr.contains("JSON"),
        "ADR-063 must anchor the existing JSON artifact/event stance"
    );
    assert!(
        adr.contains("TOML"),
        "ADR-063 must anchor the existing TOML manifest stance"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("capnproto-rust"),
        "README must mention the current capnproto-rust fit stance"
    );
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

#[test]
fn remote_model_save_load_fit_is_documented() {
    let schema_text = fs::read_to_string(fixture_path("remote_artifact_locator.schema.json"))
        .expect("read remote locator schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/remote-artifact-locator/v1")
    );
    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("schema.required must be array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = ["schema_version", "provider", "locator", "mode"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-051"),
        "architecture decisions index must link ADR-051"
    );
    assert!(
        architecture.contains("provider-neutral"),
        "architecture must mention the provider-neutral remote locator stance"
    );

    let adr = repo_file("docs/adr/051-remote-model-save-load-fit.md");
    for needle in ["provider", "locator", "mode", "storage SDK"] {
        assert!(
            adr.contains(needle),
            "ADR-051 must mention `{needle}` as part of the remote fit decision"
        );
    }

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("remote locator contract"),
        "reference index must mention the remote save/load locator stance"
    );
}
