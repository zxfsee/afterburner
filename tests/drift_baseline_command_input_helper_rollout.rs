use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn drift_baseline_family_uses_shared_command_input_helpers() {
    for path in [
        "src/cmd_drift_baseline.rs",
        "src/cmd_drift_baseline_approval.rs",
        "src/cmd_drift_baseline_bundle.rs",
        "src/cmd_drift_baseline_checkpoint.rs",
        "src/cmd_drift_baseline_handoff.rs",
        "src/cmd_drift_baseline_history.rs",
        "src/cmd_drift_baseline_pointer.rs",
        "src/cmd_drift_baseline_refresh.rs",
        "src/cmd_drift_baseline_rollback.rs",
        "src/cmd_drift_baseline_supersession.rs",
        "src/cmd_drift_baseline_transport_locator.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
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
