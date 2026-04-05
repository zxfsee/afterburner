use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn distributed_runtime_capability_surface_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-041"),
        "architecture decisions index must link ADR-041"
    );
    let adr = repo_file("docs/adr/041-distributed-runtime-capability-surface.md");
    for needle in [
        "single-device execution only",
        "data parallel execution",
        "ZeRO-1/2/3",
        "tensor parallelism",
        "pipeline parallelism",
        "sequence/context parallelism",
        "expert parallelism",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-041 must mention `{needle}` as part of the capability surface decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    for needle in [
        "single-device execution",
        "First candidate expansion: DP",
        "ZeRO-1/2/3",
        "TP/PP/SP-CP/EP",
    ] {
        assert!(
            readme.contains(needle),
            "reference index must document the current capability surface `{needle}`"
        );
    }
}
