#![recursion_limit = "256"]

pub use afterburner_core::{data, preprocess};

pub mod command_artifacts;
pub mod command_input_json;
pub mod command_reconciliation;
pub mod infer;
pub mod manifest;
pub mod model;
pub mod observability;
pub mod profiling_summary;
pub mod queue_workflow_metadata;
pub mod rollout_current_pointer;
pub mod scheduler_heartbeat_pointer_rollback_helpers;
pub mod text_pretrain;
pub mod train;
pub mod workflow_objective_lock_cli;
pub mod workflow_objective_lock_core;
pub mod workflow_process;
pub mod workflow_queue_snapshot_cli;

pub const RUNTIME_SUPPORTED_BACKENDS: &[&str] = &["cpu", "wgpu", "metal"];
