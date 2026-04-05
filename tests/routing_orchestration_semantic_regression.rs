mod support;

use support::repo_file;

#[test]
fn routing_and_orchestration_surfaces_stay_semantic_thin() {
    let main_rs = repo_file("src/main.rs");
    let cli_dispatch = repo_file("src/cli_dispatch/mod.rs");
    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");
    let justfile = repo_file("justfile");

    for text in [&main_rs, &cli_dispatch, &deploy_dispatch] {
        for forbidden in [
            "serde_json",
            "json!(",
            "emit_json_artifact_written",
            "write_json_value",
            "load_json_object(",
            "read_string(",
            "read_u64(",
            "\"schema_version\"",
            "\"artifact_role\"",
            "\"evidence_sources\"",
        ] {
            assert!(
                !text.contains(forbidden),
                "routing surface must not own payload/protocol semantics `{forbidden}`"
            );
        }
    }

    for required in [
        "cli_dispatch::run(std::env::args().skip(1))",
        "deploy::run_deploy(args)",
        "deploy::run_verify(args)",
        "deploy::run_rollback(args)",
        "deploy::run_debug_deploy(args)",
    ] {
        assert!(
            main_rs.contains(required) || cli_dispatch.contains(required),
            "routing surface must keep grouped dispatch entry `{required}`"
        );
    }

    for forbidden in [
        "jq ",
        "python -c",
        "python3 -c",
        "\"schema_version\"",
        "\"artifact_role\"",
        "\"evidence_sources\"",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must stay orchestration-only and avoid embedded protocol logic `{forbidden}`"
        );
    }
}
