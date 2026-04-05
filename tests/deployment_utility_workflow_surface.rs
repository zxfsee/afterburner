use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn deployment_utility_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "scheduler-runtime-simulate scenario:",
        "burn-bpk-migration-surface-report:",
        "distributed-load-profile addr requests concurrency latency_budget_ms_p99 error_budget_ratio:",
        "huggingface-publish request:",
        "workflow-surface-check-deployment-utility:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Deployment utility stays grouped under these operator flows:",
        "Scheduler/runtime simulation: `just scheduler-runtime-simulate`.",
        "Burn migration inventory: `just burn-bpk-migration-surface-report`, `cargo nextest run --locked --test burn_stable_release_availability`.",
        "Load and publish utilities: `just distributed-load-profile`, `afterburner deploy hf-publish --request <path>`.",
        "`just workflow-surface-check-deployment-utility` guards the grouped deployment-utility workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped deployment-utility surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just scheduler-runtime-simulate` writes `scheduler_runtime_simulation_report.json`.",
        "- `just burn-bpk-migration-surface-report` writes `burn_bpk_migration_surface_inventory.json`.",
        "- `just distributed-load-profile` writes `distributed_load_profile.json`.",
        "- `afterburner deploy hf-publish --request <path>` writes `huggingface_publish_receipt.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating deployment utility variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`scheduler_runtime_simulation_report.json`",
        "`burn_bpk_migration_surface_inventory.json`",
        "`distributed_load_profile.json`",
        "`huggingface_publish_receipt.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-utility` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped deployment-utility surface `{needle}`"
        );
    }
}
