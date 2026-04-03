use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn cleanup_family_uses_shared_command_input_helpers() {
    for path in [
        "src/cmd_cleanup_dry_run.rs",
        "src/cmd_cleanup_evidence_bundle.rs",
        "src/cmd_cleanup_execute.rs",
        "src/cmd_cleanup_inventory.rs",
        "src/cmd_cleanup_policy.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
            "fn read_array(",
            "fn read_object(",
            "fn load_history(",
            "fn history_entries_mut(",
            "fn write_json(",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} must use shared command-input helpers instead of local `{forbidden}`"
            );
        }
    }
}
