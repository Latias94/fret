# `fret-node` Fearless Refactor (v1) - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

This workstream remains the active lane for making `fret-node` the declarative-first,
controller/binding-first, editor-grade node graph surface for Fret. Recent work closed the
runtime/store contract hazards and retained canvas mirror cleanup. The current risk is no longer the
existence of the primitives; it is consumer-facing drift where docs or examples might keep teaching
older graph/view/controller triplets or direct retained authoring.

## Active Task

- Task ID: FNDX-020
- Owner: current Codex session
- Files:
  - `docs/node-graph-controlled-mode.md`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  - `ecosystem/fret-node/src/runtime/tests.rs`
  - `ecosystem/fret-node/src/ui/binding_store_sync.rs`
  - `ecosystem/fret-node/src/ui/controller_store_sync.rs`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/*`
- Validation:
  - `cargo nextest run -p fret-node controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper`
  - `cargo nextest run -p fret-node controlled_graph_can_apply_store_changes_via_callbacks`
- Status: DONE
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `docs/node-graph-controlled-mode.md` records the FNDX-020 decision: diff-first controlled sync
    remains deferred behind full-document replacement, graph-only replacement, and explicit
    transactions.
  - `ecosystem/fret-node/src/surface_policy_tests.rs` locks the public binding/controller sync
    surface against hidden `graph_diff` use or public `replace_*_with_diff` helpers.
  - Runtime controlled callback coverage still proves app-owned graph state can mirror store
    `NodeChange` / `EdgeChange` events through `apply_*_changes`.
  - Both focused validation commands passed on 2026-05-27.

## Decisions Since Last Update

- Reuse this existing workstream instead of opening a duplicate XYFlow parity lane.
- Treat `docs/workstreams/standalone/fret-node-xyflow-parity.md` as the historical parity execution
  plan and `docs/node-graph-xyflow-parity.md` as the detailed map.
- Treat the current narrow task as a consumer-surface proof: binding-first docs plus a source-policy
  gate.
- Keep diff-first controlled sync out of the public helper surface for now; require workload
  evidence before adding a `replace_*_with_diff` API.

## Blockers

- None for FNDX-020.

## Next Recommended Action

- Start FNDX-030: finish the remaining overlay/menu/toolbar policy placement decision with one
  narrow source-policy or conformance gate.
