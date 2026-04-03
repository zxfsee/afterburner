use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn scheduler_and_deployment_verification_families_use_shared_command_input_helpers() {
    let offenders = [
        "src/cmd_scheduler_heartbeat_history.rs",
        "src/cmd_scheduler_heartbeat_pointer.rs",
        "src/cmd_scheduler_heartbeat_pointer_history.rs",
        "src/cmd_scheduler_heartbeat_pointer_supersession.rs",
        "src/cmd_scheduler_heartbeat_supersession.rs",
        "src/cmd_deployment_verification_bundle.rs",
        "src/cmd_deployment_verification_bundle_history.rs",
        "src/cmd_deployment_verification_bundle_locator_history.rs",
        "src/cmd_deployment_verification_bundle_locator_pointer.rs",
        "src/cmd_deployment_verification_bundle_locator_reconcile.rs",
        "src/cmd_deployment_verification_bundle_locator_rollback.rs",
        "src/cmd_deployment_verification_bundle_locator_rollback_history.rs",
        "src/cmd_deployment_verification_bundle_rollback.rs",
        "src/cmd_deployment_verification_bundle_rollback_history.rs",
        "src/cmd_deployment_verification_bundle_rollback_reconcile.rs",
        "src/cmd_deployment_verification_bundle_rollback_supersession.rs",
        "src/cmd_deployment_verification_bundle_transport_locator.rs",
        "src/cmd_deployment_verification_bundle_transport_locator_history.rs",
        "src/cmd_deployment_verification_bundle_transport_locator_reconcile.rs",
        "src/cmd_deployment_verification_handoff.rs",
        "src/cmd_deployment_verification_handoff_history.rs",
        "src/cmd_deployment_verification_handoff_transport_locator.rs",
        "src/cmd_deployment_verification_handoff_transport_locator_history.rs",
        "src/cmd_deployment_verification_handoff_transport_locator_reconcile.rs",
        "src/cmd_deployment_verification_receipt_history.rs",
        "src/cmd_deployment_verification_receipt_locator_history.rs",
        "src/cmd_deployment_verification_receipt_locator_pointer.rs",
        "src/cmd_deployment_verification_receipt_locator_reconcile.rs",
        "src/cmd_deployment_verification_receipt_locator_rollback.rs",
        "src/cmd_deployment_verification_receipt_locator_rollback_history.rs",
        "src/cmd_deployment_verification_receipt_rollback_reconcile.rs",
        "src/cmd_deployment_verification_receipt_rollback_supersession.rs",
        "src/cmd_deployment_verification_receipt_transport_locator.rs",
        "src/cmd_deployment_verification_receipt_transport_locator_history.rs",
        "src/cmd_deployment_verification_receipt_transport_locator_reconcile.rs",
    ];

    for path in offenders {
        let text = repo_file(path);
        assert!(
            !text.contains("fn load_json_object("),
            "{path} must use shared command-input helpers instead of a local load_json_object"
        );
        assert!(
            !text.contains("fn read_string("),
            "{path} must use shared command-input helpers instead of a local read_string"
        );
        assert!(
            !text.contains("fn read_u64("),
            "{path} must use shared command-input helpers instead of a local read_u64"
        );
        assert!(
            !text.contains("fn read_optional_string("),
            "{path} must use shared command-input helpers instead of a local read_optional_string"
        );
    }
}
