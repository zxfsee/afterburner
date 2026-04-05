#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn gateway_api_inference_extension_fit_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-065"),
        "architecture decisions index must link ADR-065"
    );
    for needle in [
        "Gateway API inference extension",
        "traffic-management",
        "thin adapter target",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the Gateway API fit decision"
        );
    }

    let adr = repo_file("docs/adr/065-gateway-api-inference-extension-fit.md");
    for needle in [
        "Do not adopt the Gateway API inference extension now.",
        "traffic-management reference",
        "thin adapter target",
        "adapter-side",
        "runtime, scheduler, or deployment stack contracts",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-065 must mention `{needle}` as part of the Gateway API fit decision"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("Gateway API inference extension"),
        "README must mention the current Gateway API inference extension fit stance"
    );
}
