#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn routing_and_orchestration_guard_is_exposed_via_one_workflow_surface_check() {
    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("workflow-surface-check-routing-orchestration:"),
        "justfile must expose the routing/orchestration workflow surface check"
    );
    assert!(
        justfile.contains("--test routing_orchestration_semantic_regression")
            && justfile.contains("--test cli_dispatch_decomposition"),
        "workflow surface check must run the semantic-thin and cli-dispatch decomposition gates"
    );
}
