# `fret-node` Architecture Fearless Refactor v2 - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

The workstream is closed after completing six fearless refactor themes:

1. canonical graph mutation module,
2. store as single source of truth,
3. graph document / editor policy state split,
4. full-fidelity patch stream,
5. generic canvas mechanism extraction,
6. source-text policy test replacement.

Fresh baseline before opening:

- `cargo nextest run -p fret-node --no-default-features`: 242 passed, 0 skipped.
- `python3 tools/check_layering.py`: passed.

## Current Task

`FNAR-010` is complete.

Completed implementation task:

- `FNAR-020` - Canonical Graph Mutation Module.
- `FNAR-021` - Canonical mutation facade.
- `FNAR-030` - Store authority and document replacement events.
- `FNAR-040` - Document / editor policy state split.
- `FNAR-050` - Full-fidelity patch stream.
- `FNAR-060` - Canvas mechanism extraction.
- `FNAR-070` - Source-text policy test replacement.

`FNAR-020` implementation status:

- `apply_transaction` now applies on a scratch graph and commits atomically only after final storage
  invariants pass through `core::validate_graph_storage`.
- `GraphValidationError::PortMissingFromOwner` now closes the reverse owner-port invariant.
- `apply_op` is no longer re-exported from `ops`; the public mutation seam is transaction-level.
- Test fixtures in `rules/tests.rs` now construct owner-ordered ports through one helper.
- Status: DONE_WITH_CONCERNS. The remaining diff/invert/change-projection facade is split into
  `FNAR-021`.

`FNAR-021` implementation status:

- `GraphTransaction::{diff,apply_to,inverse,node_graph_changes}` is the public mutation facade for
  diff, application, inverse generation, and XYFlow-style change projection.
- Raw low-level `apply_transaction`, `graph_diff`, `invert_transaction`, and
  `NodeGraphChanges::from_transaction` are no longer routine public seams.
- Production store/UI commit paths publish projected changes through the transaction facade.
- Status: DONE. Fresh gates are recorded in `EVIDENCE_AND_GATES.md`.

`FNAR-030` implementation status:

- `NodeGraphStoreEvent::DocumentReplaced` carries before/after document snapshots with graph,
  graph revision, view state, and editor config.
- `NodeGraphStore::replace_document` atomically replaces graph/view/editor config, rebuilds
  lookups, advances graph revision, clears history, and emits one document replacement event.
- `NodeGraphStore::replace_graph` emits the same replacement event while preserving caller-owned
  view/history policy.
- Controller replace-document paths now call the atomic store seam directly instead of sequencing
  separate graph/view/editor updates.
- Status: DONE. Fresh gates are recorded in `EVIDENCE_AND_GATES.md`.

`FNAR-040` implementation status:

- `NodeGraphEditorStateFile` replaces the historical `NodeGraphViewStateFileV1` helper.
- Project-scoped editor-state files now use `editor_state_version`, `view_state`, and nested
  `editor_config` fields.
- Old plain-root / `state_version = 2` compatibility loaders were deleted.
- ADR/parity docs now point at `default_project_editor_state_path`.
- Status: DONE. Fresh gates are recorded in `EVIDENCE_AND_GATES.md`.

`FNAR-050` implementation status:

- `NodeGraphPatch` is the primary full-fidelity commit payload.
- `NodeGraphStoreEvent::GraphCommitted` now carries `{ patch, node_edge_changes }`.
- `DispatchOutcome`, store middleware, `install_callbacks`, and retained canvas callback glue are
  patch-first.
- `NodeGraphChanges` remains only as the explicit XyFlow-style node/edge projection adapter.
- Port-only callback coverage proves non-node/edge graph resources are visible through the patch
  even when `node_edge_changes` is empty.
- Status: DONE. Fresh gates are recorded in `EVIDENCE_AND_GATES.md`.

`FNAR-060` implementation status:

- Generic rect math helpers now live in `fret-canvas::view`.
- Generic static scene tile planning helpers now live in `fret-canvas::cache`.
- Node graph retained/declarative canvas paths delegate through thin adapters and keep node/edge
  rendering policy in `fret-node`.
- `static_scene_cache_plan.rs` was deleted after its generic helpers moved below `fret-node`.
- Status: DONE. Fresh gates are recorded in `EVIDENCE_AND_GATES.md`.

`FNAR-070` implementation status:

- `surface_policy_tests.rs` was reduced from 5101 lines to 1293 lines.
- 108 broad route-by-route implementation-shape source-policy tests were deleted.
- 41 unused `include_str!` constants were deleted after the test reduction.
- Six repeated deleted-facade scans were replaced by one narrow
  `retained_canvas_deleted_compat_facade_stays_out_of_ui_sources` guard.
- Remaining source-policy tests cover public surface shape, docs/examples, migration-ledger scans,
  and crate-private compatibility boundaries.
- Status: DONE. Fresh gates are recorded in `EVIDENCE_AND_GATES.md`.

Fresh gates are recorded in `EVIDENCE_AND_GATES.md`.

Current task:

- None. `FNAR-080` closeout is complete.

## Immediate Next Steps

No immediate continuation is required. Optional future lanes can delete the remaining retained
compatibility island or extract more generic canvas mechanisms, but those are outside this closed
six-refactor lane.

## Notes For Continuation

- This lane intentionally allows breaking compatibility.
- Do not preserve retained compatibility paths solely to avoid churn.
- Do not delete tests until the replacement seam has focused evidence.
- Keep workstream docs updated after each bounded task.
