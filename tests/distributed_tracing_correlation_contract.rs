use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn distributed_tracing_correlation_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-037"),
        "architecture decisions index must link ADR-037"
    );
    for needle in [
        "traceparent",
        "trace_id",
        "AFTERBURNER_TRACEPARENT",
        "Tokio runtime",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the tracing correlation contract"
        );
    }

    let adr = repo_file("docs/adr/037-distributed-tracing-correlation-contract.md");
    for needle in [
        "traceparent",
        "trace_id",
        "AFTERBURNER_TRACEPARENT",
        "Tokio",
        "OpenTelemetry",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-037 must mention `{needle}` as part of the tracing correlation decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("AFTERBURNER_TRACEPARENT"),
        "reference index must document the tracing correlation propagation hook"
    );
}
