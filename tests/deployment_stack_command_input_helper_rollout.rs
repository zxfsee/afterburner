mod support;

use support::repo_file;

#[test]
fn deployment_stack_launch_family_uses_shared_command_input_helpers() {
    for path in [
        "src/cmd_deployment_stack_launch_bundle.rs",
        "src/cmd_deployment_stack_launch_bundle_reconcile.rs",
        "src/cmd_deployment_stack_launch_bundle_reconciliation_history.rs",
        "src/cmd_deployment_stack_launch_handoff.rs",
        "src/cmd_deployment_stack_launch_handoff_history.rs",
        "src/cmd_deployment_stack_launch_handoff_reconcile.rs",
        "src/cmd_deployment_stack_launch_locator_history.rs",
        "src/cmd_deployment_stack_launch_locator_pointer.rs",
        "src/cmd_deployment_stack_launch_locator_reconcile.rs",
        "src/cmd_deployment_stack_launch_locator_reconciliation_history.rs",
        "src/cmd_deployment_stack_launch_receipt.rs",
        "src/cmd_deployment_stack_launch_transport_locator.rs",
        "src/cmd_deployment_stack_launch_transport_locator_history.rs",
        "src/cmd_deployment_stack_launch_transport_locator_reconcile.rs",
        "src/cmd_deployment_stack_launch_transport_locator_reconciliation_history.rs",
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
