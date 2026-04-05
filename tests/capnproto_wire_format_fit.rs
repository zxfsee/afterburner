mod support;

use support::repo_file;

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
