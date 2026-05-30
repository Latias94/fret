# `fret-node` Architecture Fearless Refactor v2 - TODO

Status: Complete
Last updated: 2026-05-27

## Cross-cutting Guardrails

- [x] Prefer deletion over compatibility shims when a historical path conflicts with the target architecture.
- [x] Keep graph document semantics headless and reusable across domains.
- [x] Keep editor policy above the headless graph document.
- [x] Keep reusable canvas mechanisms out of `fret-node` when they are not node-graph-specific.
- [x] Replace source-text policy tests with seam tests before deleting coverage.
- [x] Update ADR alignment or add an ADR note when a hard contract changes.

## M0 - Scope And Baseline

- [x] FNAR-010 [owner=planner] [deps=none] [scope=docs/workstreams/fret-node-architecture-fearless-refactor-v2,docs/node-graph-*.md]
  Goal: Freeze the six-refactor scope, baseline evidence, task ledger, and current architectural risks.
  Validation:
  - `cargo nextest run -p fret-node --no-default-features`
  - `python3 tools/check_layering.py`
  Evidence:
  - `docs/workstreams/fret-node-architecture-fearless-refactor-v2/DESIGN.md`
  - `docs/workstreams/fret-node-architecture-fearless-refactor-v2/EVIDENCE_AND_GATES.md`
  Fresh gates:
  - `cargo nextest run -p fret-node --no-default-features`: passed, 242 tests.
  - `python3 tools/check_layering.py`: passed.
  Handoff: Baseline captured; implementation starts at `FNAR-020`.

## M1 - Canonical Graph Mutation Module

- [x] FNAR-020 [owner=codex] [deps=FNAR-010] [scope=ecosystem/fret-node/src/core/validate.rs,ecosystem/fret-node/src/ops,ecosystem/fret-node/src/rules/tests.rs]
  Goal: Land the canonical transaction apply/storage-invariant seam.
  Validation:
  - `cargo nextest run -p fret-node graph_diff`
  - `cargo nextest run -p fret-node changes_to_transaction_is_reversible_and_applicable`
  - `cargo nextest run -p fret-node apply_node_changes`
  Review: `review-workstream` before accepting this slice.
  Evidence:
  - mutation module code paths
  - new or updated tests proving node-with-ports and port-owner invariants
  Fresh gates:
  - `cargo nextest run -p fret-node apply_transaction_rejects_`: passed, 2 tests.
  - `cargo nextest run -p fret-node graph_diff`: passed, 8 tests.
  - `cargo nextest run -p fret-node changes_to_transaction_is_reversible_and_applicable`: passed.
  - `cargo nextest run -p fret-node apply_node_changes`: passed.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 245 tests.
  - `cargo nextest run -p fret-node`: passed, 557 tests.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  - `cargo fmt --check`: passed.
  - `python3 tools/check_layering.py`: passed.
  Handoff: DONE_WITH_CONCERNS. The transaction apply/storage seam is complete; the broader mutation
  facade for diff/invert/change projection is split into `FNAR-021`.

- [x] FNAR-021 [owner=codex] [deps=FNAR-020] [scope=ecosystem/fret-node/src/ops,ecosystem/fret-node/src/runtime/changes.rs,ecosystem/fret-node/src/runtime/store.rs]
  Goal: Finish the canonical mutation facade around diff, inverse generation, and change projection so callers do not coordinate raw op ordering by hand.
  Validation:
  - `cargo nextest run -p fret-node graph_diff`
  - `cargo nextest run -p fret-node changes_from_transaction`
  - `cargo nextest run -p fret-node changes_to_transaction_is_reversible_and_applicable`
  - `cargo nextest run -p fret-node store_dispatch_changes_records_history_and_supports_undo`
  Review: `review-workstream` before accepting this slice.
  Evidence:
  - mutation facade code paths
  - tests proving diff/invert/projection roundtrip through the public seam
  Fresh gates:
  - `cargo nextest run -p fret-node graph_transaction_facade_diff_apply_and_inverse_roundtrip`: passed.
  - `cargo nextest run -p fret-node changes_from_transaction_maps_ops`: passed.
  - `cargo nextest run -p fret-node changes_to_transaction_is_reversible_and_applicable`: passed.
  - `cargo nextest run -p fret-node store_dispatch_changes_records_history_and_supports_undo`: passed.
  - `cargo nextest run -p fret-node store_dispatch_pipeline_publishes_coherent_commit_state`: passed.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 246 tests.
  - `cargo nextest run -p fret-node`: passed, 558 tests.
  - `cargo fmt --check`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  Handoff: DONE. Public mutation coordination now routes through `GraphTransaction::{diff,apply_to,inverse,node_graph_changes}`; raw apply/diff/invert helpers and transaction-to-change projection are internal seams.

## M2 - Store Authority And Document Replacement Events

- [x] FNAR-030 [owner=codex] [deps=FNAR-021] [scope=ecosystem/fret-node/src/runtime,ecosystem/fret-node/src/ui/binding*,ecosystem/fret-node/src/ui/controller*]
  Goal: Make `NodeGraphStore` the authoritative source for graph, view state, editor config, document replacement, and graph revisions.
  Validation:
  - `cargo nextest run -p fret-node store_`
  - `cargo nextest run -p fret-node controlled_graph_can_apply_store_changes_via_callbacks`
  - focused document replacement event test
  Review: `review-workstream` before accepting this slice.
  Evidence:
  - store event model
  - binding/controller sync simplification
  Fresh gates:
  - `cargo nextest run -p fret-node store_`: passed, 51 tests.
  - `cargo nextest run -p fret-node install_callbacks`: passed, 3 tests.
  - `cargo nextest run -p fret-node store_replace_document_emits_single_document_event_and_clears_history`: passed.
  - `cargo nextest run -p fret-node store_replace_graph_emits_document_event_and_preserves_history_policy`: passed.
  - `cargo nextest run -p fret-node controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper public_node_graph_guides_teach_binding_first_surface`: passed, 2 tests.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 248 tests.
  - `cargo nextest run -p fret-node`: passed, 560 tests.
  - `cargo fmt --check`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  Handoff: DONE. Document replacement is now an atomic store operation with one `DocumentReplaced` event; controller/binding replace-document paths call the store seam directly.

## M3 - Document / Editor Policy State Split

- [x] FNAR-040 [owner=codex] [deps=FNAR-030] [scope=ecosystem/fret-node/src/core,ecosystem/fret-node/src/io,ecosystem/fret-node/src/interaction,ecosystem/fret-node/src/profile]
  Goal: Separate semantic graph document state from editor policy, view, and derived UI state.
  Validation:
  - `cargo nextest run -p fret-node core::`
  - `cargo nextest run -p fret-node io::`
  - serialization migration tests if persisted shape changes
  Review: `review-workstream` before accepting this slice.
  Evidence:
  - split model types
  - persistence tests
  Fresh gates:
  - `cargo nextest run -p fret-node io::`: passed, 5 tests.
  - `cargo nextest run -p fret-node core::`: passed, 13 tests.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 247 tests.
  - `cargo nextest run -p fret-node`: passed, 559 tests.
  - `cargo fmt --check`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  Handoff: DONE. `NodeGraphEditorStateFile` replaces the historical view-state file helper and
  persists `view_state` separately from nested `editor_config`; old compatibility loader paths were
  deleted because this lane does not preserve historical local-state formats.

## M4 - Full-Fidelity Patch Stream

- [x] FNAR-050 [owner=codex] [deps=FNAR-021,FNAR-030] [scope=ecosystem/fret-node/src/runtime/events.rs,ecosystem/fret-node/src/runtime/callbacks.rs,ecosystem/fret-node/src/runtime/middleware.rs,ecosystem/fret-node/src/runtime/changes.rs]
  Goal: Make full-fidelity patch events primary and expose XYFlow-style node/edge changes as a lossy adapter.
  Validation:
  - `cargo nextest run -p fret-node changes_from_transaction`
  - `cargo nextest run -p fret-node install_callbacks`
  - focused callback test for ports/groups/symbols/imports or document replacement
  Review: `review-workstream` before accepting this slice.
  Evidence:
  - patch stream event tests
  - controlled-mode docs
  Fresh gates:
  - `cargo nextest run -p fret-node changes_from_transaction install_callbacks store_dispatch_pipeline_publishes_coherent_commit_state`: passed, 10 tests.
  - `cargo nextest run -p fret-node store_dispatch_pipeline_publishes_coherent_commit_state install_callbacks_receives_full_patch_for_port_only_commits`: passed, 2 tests.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 248 tests.
  - `cargo nextest run -p fret-node`: passed, 560 tests.
  - `cargo fmt --check`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  Handoff: DONE. `NodeGraphPatch` is now the primary commit payload; `NodeGraphChanges` remains
  as the explicitly lossy `node_edge_changes` adapter for XyFlow-style arrays.

## M5 - Canvas Mechanism Extraction

- [x] FNAR-060 [owner=codex] [deps=FNAR-030] [scope=ecosystem/fret-node/src/ui/canvas,ecosystem/fret-node/src/ui/declarative/paint_only,ecosystem/fret-canvas]
  Goal: Move reusable pan/zoom, spatial query, gesture session, culling/cache, and route helpers below `fret-node` where they are not node-graph-specific.
  Validation:
  - `cargo nextest run -p fret-node --features compat-retained-canvas`
  - `cargo nextest run -p fret-canvas`
  - `python3 tools/check_layering.py`
  Review: `review-workstream` before accepting this slice.
  Evidence:
  - extracted `fret-canvas` module seams
  - node graph adapters
  Fresh gates:
  - `cargo nextest run -p fret-canvas`: passed, 70 tests.
  - `cargo nextest run -p fret-node --features compat-retained-canvas`: passed, 1203 tests.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 248 tests.
  - `cargo nextest run -p fret-node`: passed, 560 tests.
  - `cargo fmt --check`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  - `cargo clippy -p fret-canvas --all-targets -- -D warnings`: passed.
  Handoff: DONE. Domain-neutral rect and static scene tile planning helpers now live in
  `fret-canvas`; `fret-node` keeps node/edge-specific policy through thin adapters.

## M6 - Replace Source-Text Policy Tests With Seam Tests

- [x] FNAR-070 [owner=codex] [deps=FNAR-021,FNAR-030,FNAR-050,FNAR-060] [scope=ecosystem/fret-node/src/surface_policy_tests.rs,ecosystem/fret-node/src/ui/**/tests*.rs,docs/node-graph-*.md]
  Goal: Reduce source-text policy tests to narrow layering guards and replace broad implementation-shape assertions with seam tests.
  Validation:
  - `cargo nextest run -p fret-node`
  - `cargo nextest run -p fret-node --no-default-features`
  - focused tests for public guide, transaction, callback, and diagnostics seams
  Review: `review-workstream` before accepting this slice.
  Evidence:
  - deleted or reduced source-policy tests
  - replacement behavior gates
  Fresh gates:
  - `cargo nextest run -p fret-node --no-default-features retained_canvas_deleted_compat_facade_stays_out_of_ui_sources retained_bridge_source_usage_stays_on_the_migration_ledger`: passed, 2 tests.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 134 tests.
  - `cargo nextest run -p fret-node`: passed, 446 tests.
  - `cargo nextest run -p fret-node --features compat-retained-canvas`: passed, 1089 tests.
  - `cargo fmt --check`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  Handoff: DONE. Broad route-by-route source-text guards were deleted in favor of package compile
  gates and behavior seam tests; remaining source-policy tests are narrow public-surface,
  documentation, migration-ledger, or crate-boundary guards.

## M7 - Closeout

- [x] FNAR-080 [owner=planner] [deps=FNAR-070] [scope=docs/workstreams/fret-node-architecture-fearless-refactor-v2,docs/node-graph-roadmap.md,docs/node-graph-xyflow-parity.md]
  Goal: Verify, review, document final architecture, and close or split follow-ons.
  Validation:
  - `cargo fmt --check`
  - `cargo nextest run -p fret-node`
  - `python3 tools/check_layering.py`
  - broader workspace gate if scope crossed crates significantly
  Review: `verify-rust-workstream` then `close-workstream`.
  Evidence:
  - `EVIDENCE_AND_GATES.md`
  - `WORKSTREAM.json`
  - `HANDOFF.md`
  Fresh gates:
  - `cargo fmt --check`: passed.
  - `cargo nextest run -p fret-node`: passed, 446 tests.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 134 tests.
  - `cargo nextest run -p fret-node --features compat-retained-canvas`: passed, 1089 tests.
  - `cargo nextest run -p fret-canvas`: passed, 70 tests.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  - `cargo clippy -p fret-canvas --all-targets -- -D warnings`: passed.
  Handoff: COMPLETE. All six target refactors landed with fresh evidence; no required follow-on is
  needed to close this lane.
