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

- Task ID: FNDX-010
- Owner: current Codex session
- Files:
  - `docs/node-graph-how-to-build-like-xyflow.md`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/*`
- Validation:
  - `cargo nextest run -p fret-node public_node_graph_guides_teach_binding_first_surface`
  - `cargo fmt --check`
- Status: DONE
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `docs/node-graph-how-to-build-like-xyflow.md` now teaches binding-first integration.
  - `ecosystem/fret-node/src/surface_policy_tests.rs` now locks the public guide against stale
    graph/view/model triplets or direct retained-canvas teaching.
  - Both validation commands passed on 2026-05-27.

## Decisions Since Last Update

- Reuse this existing workstream instead of opening a duplicate XYFlow parity lane.
- Treat `docs/workstreams/standalone/fret-node-xyflow-parity.md` as the historical parity execution
  plan and `docs/node-graph-xyflow-parity.md` as the detailed map.
- Treat the current narrow task as a consumer-surface proof: binding-first docs plus a source-policy
  gate.

## Blockers

- None for FNDX-010.

## Next Recommended Action

- Decide whether FNDX-020 should address diff-first controlled sync now, or whether FNDX-030
  overlay/menu/toolbar policy placement has higher payoff for the next bounded task.
