#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

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
        reference.contains("Burn stable release availability gate"),
        "reference index must mention the Burn stable release availability gate"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("cargo nextest run --locked --test burn_stable_release_availability"),
        "workflow reference must mention the Burn stable release availability gate command"
    );
}
