use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Map, Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/simulation/scheduler_runtime_simulation_report.json";

#[derive(Debug)]
enum SchedulerRuntimeSimulationError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for SchedulerRuntimeSimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SchedulerRuntimeSimulationError {}

impl From<std::io::Error> for SchedulerRuntimeSimulationError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for SchedulerRuntimeSimulationError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize scheduler/runtime simulation report json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    scenario: PathBuf,
    out_path: PathBuf,
}

#[derive(Debug)]
struct SimulationScenario {
    schema_version: String,
    scenario_name: String,
    inventory: Inventory,
    lease: Lease,
    layout: Layout,
    lifecycle_messages: Vec<LifecycleMessage>,
    checkpoint_restart: CheckpointRestart,
    artifact_plan: Vec<ArtifactPlanItem>,
}

#[derive(Debug)]
struct Inventory {
    node_count: u64,
    gpus_per_node: u64,
}

#[derive(Debug)]
struct Lease {
    job_id: String,
    lease_id: String,
    node_ids: Vec<String>,
    gpu_ids: Vec<String>,
    checkpoint_mode: String,
}

#[derive(Debug)]
struct Layout {
    world_size: u64,
    parallelism: LayoutParallelism,
}

#[derive(Debug)]
struct LayoutParallelism {
    dp: u64,
    tp: u64,
    pp: u64,
    sp_cp: u64,
    ep: u64,
    gas: u64,
    microbatch_size: u64,
    zero_stage: u64,
}

#[derive(Debug)]
struct LifecycleMessage {
    direction: String,
    kind: String,
    job_id: String,
    lease_id: String,
}

#[derive(Debug)]
struct CheckpointRestart {
    checkpoint_group: String,
    restart_after_kind: String,
    expected_resume_kind: String,
}

#[derive(Debug)]
struct ArtifactPlanItem {
    kind: String,
    event: String,
    path: String,
    after_kind: String,
}

pub fn run<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args: Vec<String> = args.collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return 0;
    }

    match run_inner(args.into_iter()) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            2
        }
    }
}

fn run_inner<I>(args: I) -> Result<(), SchedulerRuntimeSimulationError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let scenario = load_scenario(args.scenario.as_path())?;
    validate_scenario(&scenario, args.scenario.as_path())?;

    let lifecycle = lifecycle_report(&scenario);
    let lease_transitions = lease_transitions(&scenario);
    let layout_feasibility = layout_feasibility(&scenario);
    let checkpoint_restart = checkpoint_restart_report(&scenario);
    let artifact_emission = artifact_emission_report(&scenario);

    let report = json!({
        "schema_version": "1",
        "scenario_name": scenario.scenario_name,
        "scenario_path": args.scenario.display().to_string(),
        "tick_count": scenario.lifecycle_messages.len(),
        "scheduler_lifecycle": lifecycle,
        "lease_transitions": lease_transitions,
        "layout_feasibility": layout_feasibility,
        "checkpoint_restart": checkpoint_restart,
        "artifact_emission": artifact_emission,
    });

    write_json_value(&args.out_path, &report)?;
    emit_json_artifact_written(
        "debug_cli",
        "scheduler_runtime_simulation_report_written",
        "report_path",
        &args.out_path,
        "report",
        report,
    );
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, SchedulerRuntimeSimulationError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut scenario = None::<PathBuf>;
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" => {
                scenario = Some(PathBuf::from(parse_value::<String, _>(
                    &mut args,
                    "--scenario",
                )?))
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--scenario=") => {
                scenario = Some(PathBuf::from(
                    arg.trim_start_matches("--scenario=").to_string(),
                ))
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(SchedulerRuntimeSimulationError::InvalidArg(format!(
                    "unknown argument for debug simulate-scheduler-runtime: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        scenario: scenario.ok_or_else(|| {
            SchedulerRuntimeSimulationError::InvalidArg(format!(
                "missing value for --scenario\n{}",
                usage()
            ))
        })?,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, SchedulerRuntimeSimulationError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        SchedulerRuntimeSimulationError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        SchedulerRuntimeSimulationError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn load_scenario(path: &Path) -> Result<SimulationScenario, SchedulerRuntimeSimulationError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        SchedulerRuntimeSimulationError::Parse(format!(
            "parse scheduler/runtime simulation scenario `{}`: {err}",
            path.display()
        ))
    })?;
    let object = value.as_object().ok_or_else(|| {
        SchedulerRuntimeSimulationError::Parse(format!(
            "scheduler/runtime simulation scenario `{}` must be an object",
            path.display()
        ))
    })?;
    simulation_scenario_from_object(object, path)
}

fn validate_scenario(
    scenario: &SimulationScenario,
    path: &Path,
) -> Result<(), SchedulerRuntimeSimulationError> {
    if scenario.schema_version != "1" {
        return Err(SchedulerRuntimeSimulationError::Parse(format!(
            "scheduler/runtime simulation scenario `{}` must use schema_version `1`",
            path.display()
        )));
    }
    if scenario.lifecycle_messages.is_empty() {
        return Err(SchedulerRuntimeSimulationError::Parse(format!(
            "scheduler/runtime simulation scenario `{}` must contain at least one lifecycle message",
            path.display()
        )));
    }
    for message in &scenario.lifecycle_messages {
        if message.job_id != scenario.lease.job_id || message.lease_id != scenario.lease.lease_id {
            return Err(SchedulerRuntimeSimulationError::Parse(format!(
                "scheduler/runtime simulation scenario `{}` lifecycle messages must match lease job_id and lease_id",
                path.display()
            )));
        }
    }
    Ok(())
}

fn simulation_scenario_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<SimulationScenario, SchedulerRuntimeSimulationError> {
    Ok(SimulationScenario {
        schema_version: read_string(object, "schema_version", path, "simulation scenario")?,
        scenario_name: read_string(object, "scenario_name", path, "simulation scenario")?,
        inventory: inventory_from_object(
            &read_object(object, "inventory", path, "simulation scenario")?,
            path,
        )?,
        lease: lease_from_object(
            &read_object(object, "lease", path, "simulation scenario")?,
            path,
        )?,
        layout: layout_from_object(
            &read_object(object, "layout", path, "simulation scenario")?,
            path,
        )?,
        lifecycle_messages: read_array(object, "lifecycle_messages", path, "simulation scenario")?
            .iter()
            .map(|value| {
                let object = value.as_object().ok_or_else(|| {
                    SchedulerRuntimeSimulationError::Parse(format!(
                        "simulation scenario `{}` lifecycle_messages must contain only objects",
                        path.display()
                    ))
                })?;
                lifecycle_message_from_object(object, path)
            })
            .collect::<Result<Vec<_>, _>>()?,
        checkpoint_restart: checkpoint_restart_from_object(
            &read_object(object, "checkpoint_restart", path, "simulation scenario")?,
            path,
        )?,
        artifact_plan: read_array(object, "artifact_plan", path, "simulation scenario")?
            .iter()
            .map(|value| {
                let object = value.as_object().ok_or_else(|| {
                    SchedulerRuntimeSimulationError::Parse(format!(
                        "simulation scenario `{}` artifact_plan must contain only objects",
                        path.display()
                    ))
                })?;
                artifact_plan_item_from_object(object, path)
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn inventory_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<Inventory, SchedulerRuntimeSimulationError> {
    Ok(Inventory {
        node_count: read_u64(object, "node_count", path, "simulation inventory")?,
        gpus_per_node: read_u64(object, "gpus_per_node", path, "simulation inventory")?,
    })
}

fn lease_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<Lease, SchedulerRuntimeSimulationError> {
    Ok(Lease {
        job_id: read_string(object, "job_id", path, "simulation lease")?,
        lease_id: read_string(object, "lease_id", path, "simulation lease")?,
        node_ids: read_string_array(object, "node_ids", path, "simulation lease")?,
        gpu_ids: read_string_array(object, "gpu_ids", path, "simulation lease")?,
        checkpoint_mode: read_string(object, "checkpoint_mode", path, "simulation lease")?,
    })
}

fn layout_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<Layout, SchedulerRuntimeSimulationError> {
    Ok(Layout {
        world_size: read_u64(object, "world_size", path, "simulation layout")?,
        parallelism: layout_parallelism_from_object(
            &read_object(object, "parallelism", path, "simulation layout")?,
            path,
        )?,
    })
}

fn layout_parallelism_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<LayoutParallelism, SchedulerRuntimeSimulationError> {
    Ok(LayoutParallelism {
        dp: read_u64(object, "dp", path, "simulation parallelism")?,
        tp: read_u64(object, "tp", path, "simulation parallelism")?,
        pp: read_u64(object, "pp", path, "simulation parallelism")?,
        sp_cp: read_u64(object, "sp_cp", path, "simulation parallelism")?,
        ep: read_u64(object, "ep", path, "simulation parallelism")?,
        gas: read_u64(object, "gas", path, "simulation parallelism")?,
        microbatch_size: read_u64(object, "microbatch_size", path, "simulation parallelism")?,
        zero_stage: read_u64(object, "zero_stage", path, "simulation parallelism")?,
    })
}

fn lifecycle_message_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<LifecycleMessage, SchedulerRuntimeSimulationError> {
    Ok(LifecycleMessage {
        direction: read_string(object, "direction", path, "simulation lifecycle message")?,
        kind: read_string(object, "kind", path, "simulation lifecycle message")?,
        job_id: read_string(object, "job_id", path, "simulation lifecycle message")?,
        lease_id: read_string(object, "lease_id", path, "simulation lifecycle message")?,
    })
}

fn checkpoint_restart_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<CheckpointRestart, SchedulerRuntimeSimulationError> {
    Ok(CheckpointRestart {
        checkpoint_group: read_string(
            object,
            "checkpoint_group",
            path,
            "simulation checkpoint_restart",
        )?,
        restart_after_kind: read_string(
            object,
            "restart_after_kind",
            path,
            "simulation checkpoint_restart",
        )?,
        expected_resume_kind: read_string(
            object,
            "expected_resume_kind",
            path,
            "simulation checkpoint_restart",
        )?,
    })
}

fn artifact_plan_item_from_object(
    object: &Map<String, Value>,
    path: &Path,
) -> Result<ArtifactPlanItem, SchedulerRuntimeSimulationError> {
    Ok(ArtifactPlanItem {
        kind: read_string(object, "kind", path, "simulation artifact_plan item")?,
        event: read_string(object, "event", path, "simulation artifact_plan item")?,
        path: read_string(object, "path", path, "simulation artifact_plan item")?,
        after_kind: read_string(object, "after_kind", path, "simulation artifact_plan item")?,
    })
}

fn lifecycle_report(scenario: &SimulationScenario) -> Value {
    let messages = scenario
        .lifecycle_messages
        .iter()
        .enumerate()
        .map(|(step_index, message)| {
            let accepted = matches!(
                (message.direction.as_str(), message.kind.as_str()),
                ("scheduler_to_runtime", "START" | "STOP" | "KILL")
                    | (
                        "runtime_to_scheduler",
                        "READY" | "CHECKPOINTED" | "FAILED" | "HEARTBEAT"
                    )
            );
            json!({
                "step_index": step_index as u64,
                "direction": message.direction,
                "kind": message.kind,
                "accepted": accepted,
            })
        })
        .collect::<Vec<_>>();

    let final_kind = scenario
        .lifecycle_messages
        .last()
        .map(|message| message.kind.as_str())
        .unwrap_or("UNKNOWN");

    json!({
        "messages": messages,
        "final_kind": final_kind,
    })
}

fn lease_transitions(scenario: &SimulationScenario) -> Value {
    let mut current_state = "pending".to_string();
    let transitions = scenario
        .lifecycle_messages
        .iter()
        .enumerate()
        .map(|(step_index, message)| {
            let next_state = match message.kind.as_str() {
                "START" => "leased",
                "READY" | "HEARTBEAT" => "running",
                "CHECKPOINTED" => "checkpointed",
                "FAILED" => "failed",
                "STOP" | "KILL" => "stopped",
                _ => current_state.as_str(),
            }
            .to_string();
            let transition = json!({
                "step_index": step_index as u64,
                "trigger_kind": message.kind,
                "from_state": current_state,
                "to_state": next_state,
            });
            current_state = transition["to_state"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            transition
        })
        .collect::<Vec<_>>();

    Value::Array(transitions)
}

fn layout_feasibility(scenario: &SimulationScenario) -> Value {
    let total_gpus = scenario.inventory.node_count * scenario.inventory.gpus_per_node;
    let parallelism_product = scenario.layout.parallelism.dp
        * scenario.layout.parallelism.tp
        * scenario.layout.parallelism.pp
        * scenario.layout.parallelism.sp_cp
        * scenario.layout.parallelism.ep;
    let checks = vec![
        json!({
            "name": "available_gpus_cover_world_size",
            "passed": total_gpus >= scenario.layout.world_size,
            "detail": format!("available_gpus={} world_size={}", total_gpus, scenario.layout.world_size),
        }),
        json!({
            "name": "world_size_matches_parallelism_product",
            "passed": scenario.layout.world_size == parallelism_product,
            "detail": format!("world_size={} parallelism_product={}", scenario.layout.world_size, parallelism_product),
        }),
        json!({
            "name": "lease_gpu_count_covers_world_size",
            "passed": (scenario.lease.gpu_ids.len() as u64) >= scenario.layout.world_size,
            "detail": format!("lease_gpu_count={} world_size={}", scenario.lease.gpu_ids.len(), scenario.layout.world_size),
        }),
        json!({
            "name": "lease_node_count_covers_inventory",
            "passed": !scenario.lease.node_ids.is_empty() && (scenario.lease.node_ids.len() as u64) <= scenario.inventory.node_count,
            "detail": format!("lease_node_count={} inventory_node_count={}", scenario.lease.node_ids.len(), scenario.inventory.node_count),
        }),
    ];
    let feasible = checks
        .iter()
        .all(|check| check["passed"].as_bool().unwrap_or(false));

    json!({
        "schema_version": "1",
        "node_count": scenario.inventory.node_count,
        "gpus_per_node": scenario.inventory.gpus_per_node,
        "world_size": scenario.layout.world_size,
        "parallelism": {
            "dp": scenario.layout.parallelism.dp,
            "tp": scenario.layout.parallelism.tp,
            "pp": scenario.layout.parallelism.pp,
            "sp_cp": scenario.layout.parallelism.sp_cp,
            "ep": scenario.layout.parallelism.ep,
            "gas": scenario.layout.parallelism.gas,
            "microbatch_size": scenario.layout.parallelism.microbatch_size,
            "zero_stage": scenario.layout.parallelism.zero_stage,
        },
        "feasible": feasible,
        "checks": checks,
    })
}

fn checkpoint_restart_report(scenario: &SimulationScenario) -> Value {
    let checkpointed_index = scenario
        .lifecycle_messages
        .iter()
        .position(|message| message.kind == scenario.checkpoint_restart.restart_after_kind);
    let checkpointed_observed = checkpointed_index.is_some();
    let restart_index = checkpointed_index.and_then(|index| {
        scenario
            .lifecycle_messages
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, message)| message.kind == "START")
            .map(|(index, _)| index)
    });
    let resume_index = restart_index.and_then(|index| {
        scenario
            .lifecycle_messages
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, message)| message.kind == scenario.checkpoint_restart.expected_resume_kind)
            .map(|(index, _)| index)
    });
    let restart_permitted = scenario.lease.checkpoint_mode == "cooperative";
    let checks = vec![
        json!({
            "name": "checkpointed_observed",
            "passed": checkpointed_observed,
            "detail": format!("restart_after_kind={}", scenario.checkpoint_restart.restart_after_kind),
        }),
        json!({
            "name": "restart_permitted",
            "passed": restart_permitted,
            "detail": format!("checkpoint_mode={}", scenario.lease.checkpoint_mode),
        }),
        json!({
            "name": "restart_observed",
            "passed": restart_index.is_some(),
            "detail": "observed a START after checkpoint".to_string(),
        }),
        json!({
            "name": "expected_resume_observed",
            "passed": resume_index.is_some(),
            "detail": format!("expected_resume_kind={}", scenario.checkpoint_restart.expected_resume_kind),
        }),
    ];

    json!({
        "checkpoint_group": scenario.checkpoint_restart.checkpoint_group,
        "checkpointed_observed": checkpointed_observed,
        "restart_permitted": restart_permitted,
        "restart_observed": restart_index.is_some(),
        "expected_resume_kind": scenario.checkpoint_restart.expected_resume_kind,
        "expected_resume_observed": resume_index.is_some(),
        "checks": checks,
    })
}

fn artifact_emission_report(scenario: &SimulationScenario) -> Value {
    let expected = scenario
        .artifact_plan
        .iter()
        .map(|item| {
            let emitted_after_step = scenario
                .lifecycle_messages
                .iter()
                .enumerate()
                .find(|(_, message)| message.kind == item.after_kind)
                .map(|(index, _)| index as u64);
            json!({
                "kind": item.kind,
                "event": item.event,
                "path": item.path,
                "after_kind": item.after_kind,
                "emitted": emitted_after_step.is_some(),
                "emitted_after_step": emitted_after_step,
            })
        })
        .collect::<Vec<_>>();
    let all_expected_emitted = expected
        .iter()
        .all(|item| item["emitted"].as_bool().unwrap_or(false));

    json!({
        "expected": expected,
        "all_expected_emitted": all_expected_emitted,
    })
}

fn read_object(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<Map<String, Value>, SchedulerRuntimeSimulationError> {
    object
        .get(key)
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| {
            SchedulerRuntimeSimulationError::Parse(format!(
                "{kind} `{}` missing object field `{key}`",
                path.display()
            ))
        })
}

fn read_array<'a>(
    object: &'a Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<&'a Vec<Value>, SchedulerRuntimeSimulationError> {
    object.get(key).and_then(Value::as_array).ok_or_else(|| {
        SchedulerRuntimeSimulationError::Parse(format!(
            "{kind} `{}` missing array field `{key}`",
            path.display()
        ))
    })
}

fn read_string(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<String, SchedulerRuntimeSimulationError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            SchedulerRuntimeSimulationError::Parse(format!(
                "{kind} `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

fn read_u64(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<u64, SchedulerRuntimeSimulationError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        SchedulerRuntimeSimulationError::Parse(format!(
            "{kind} `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

fn read_string_array(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<Vec<String>, SchedulerRuntimeSimulationError> {
    read_array(object, key, path, kind)?
        .iter()
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                SchedulerRuntimeSimulationError::Parse(format!(
                    "{kind} `{}` field `{key}` must contain only strings",
                    path.display()
                ))
            })
        })
        .collect()
}

fn usage() -> &'static str {
    "usage: afterburner debug simulate-scheduler-runtime --scenario PATH [--out PATH]"
}
