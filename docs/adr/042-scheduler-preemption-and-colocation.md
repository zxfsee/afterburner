# ADR-042: Scheduler Preemption And Colocation

## Context

The repo now separates distributed runtime, scheduler, and platform layers, but
it still needs one explicit scheduler stance for small Rust-first clusters with
mostly single-GPU jobs. Without that stance, scheduler work can drift toward
unrealistic assumptions about transparent GPU task suspension or blur
checkpoint-based preemption together with GPU sharing.

For modest clusters, the practical scheduler options are narrower:

- queueing and priority without preemption,
- cooperative preemption through checkpoint/resume,
- limited GPU colocation for known-compatible workloads.

## Decision

The scheduler stance is intentionally conservative.

- Default local-cluster mode is queueing plus priority, with no transparent GPU
  preemption assumption.
- Supported preemption model is cooperative checkpoint/resume only.
  - The scheduler may request stop/resume through the lifecycle boundary.
  - The runtime owns safe checkpoint points and restore correctness.
  - Without checkpoint support, preemption is not a supported feature.
- GPU colocation is a separate resource-management mode, not a form of
  transparent preemption.
  - If added later, it should be limited to explicitly approved workload
    classes and measured for interference.
- Do not assume UVM/MPS-style transparent pause/resume is an available or
  reliable scheduler primitive.

Current stance: this ADR defines scheduler policy boundaries only. It does not
add cooperative preemption or colocation logic yet.

## Consequences

- Scheduler work stays realistic for modest clusters instead of depending on
  unsupported transparent GPU suspension semantics.
- Runtime and scheduler ownership remain clean: scheduler owns policy and
  lifecycle; runtime owns checkpoint correctness.
- Future preemption and colocation work can be added as separate follow-on
  items instead of being conflated into one scheduler feature.
