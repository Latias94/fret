# `fret-node` Architecture Fearless Refactor v2 - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Baseline Evidence

Fresh baseline before opening this lane:

```bash
cargo nextest run -p fret-node --no-default-features
```

Result on 2026-05-27: 242 tests passed, 0 skipped.

Layering baseline:

```bash
python3 tools/check_layering.py
```

Result on 2026-05-27: passed.

## Gate Set

### Baseline / Headless Gate

```bash
cargo nextest run -p fret-node --no-default-features
```

Proves the headless and default-no-UI contract remains coherent during mutation/store work.

### Targeted Mutation Gate

```bash
cargo nextest run -p fret-node graph_diff
cargo nextest run -p fret-node changes_to_transaction_is_reversible_and_applicable
cargo nextest run -p fret-node apply_node_changes
```

Proves mutation, diff, inverse, and controlled-change adapters remain coherent.

### Runtime / Store Gate

```bash
cargo nextest run -p fret-node store_
cargo nextest run -p fret-node install_callbacks
cargo nextest run -p fret-node controlled_graph_can_apply_store_changes_via_callbacks
```

Proves store authority, callback delivery, and controlled-mode synchronization.

### UI / Compatibility Gate

```bash
cargo nextest run -p fret-node --features compat-retained-canvas
cargo check -p fret-node --features compat-retained-canvas --tests
```

Proves retained compatibility tests still compile until the lane intentionally deletes or replaces
that path.

### Cross-Crate Canvas Gate

```bash
cargo nextest run -p fret-canvas
python3 tools/check_layering.py
```

Proves extracted canvas mechanisms do not violate crate layering.

### Package Gate

```bash
cargo nextest run -p fret-node
```

Proves the package remains coherent after each major slice.

### Formatting Gate

```bash
cargo fmt --check
```

Proves repository formatting is stable.

## Review Gate

Run `review-workstream` before accepting a major slice as done.

Run `verify-rust-workstream` before marking the lane complete or closing it.

## Evidence Anchors

- `docs/workstreams/fret-node-architecture-fearless-refactor-v2/DESIGN.md`
- `docs/workstreams/fret-node-architecture-fearless-refactor-v2/TODO.md`
- `docs/workstreams/fret-node-architecture-fearless-refactor-v2/MILESTONES.md`
- `docs/node-graph-roadmap.md`
- `docs/node-graph-xyflow-parity.md`
- `docs/node-graph-controlled-mode.md`
- `ecosystem/fret-node/src/core/model.rs`
- `ecosystem/fret-node/src/ops`
- `ecosystem/fret-node/src/runtime`
- `ecosystem/fret-node/src/ui`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-canvas`

## Notes

- Fresh evidence must be appended after each task lands.
- Do not record command names without the behavior they prove.
- Deleting compatibility code is allowed when replacement seams and gates are in place.

## FNAR-020 Verification - 2026-05-27

Claim: graph mutations now have a stronger canonical transaction apply seam for storage
invariants: `apply_transaction` applies on a scratch graph, validates final storage invariants via
`core::validate_graph_storage`, commits atomically on success, and low-level `apply_op` is no longer
re-exported from `ops`.

Fresh gates:

- `cargo nextest run -p fret-node apply_transaction_rejects_`: passed, 2 tests. Proves invalid
  node/port owner-storage transactions are rejected atomically.
- `cargo nextest run -p fret-node graph_diff`: passed, 8 tests. Proves existing diff-generated
  transaction ordering remains apply-safe.
- `cargo nextest run -p fret-node changes_to_transaction_is_reversible_and_applicable`: passed.
  Proves controlled change projection remains reversible and applicable.
- `cargo nextest run -p fret-node apply_node_changes`: passed. Proves node removal change helpers
  still remove owned ports and incident edges.
- `cargo nextest run -p fret-node --no-default-features`: passed, 245 tests. Proves the headless
  mutation/runtime baseline after the invariant tightening.
- `cargo nextest run -p fret-node`: passed, 557 tests. Proves default UI-enabled package behavior
  after the invariant tightening.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Gate not accepted as task evidence:

- `cargo clippy -p fret-node --all-targets -- -D warnings`: failed in pre-existing `fret-ui`
  clean-geometry lints (`enum_variant_names`, `large_enum_variant`, `collapsible_if`,
  `redundant_closure_call`) before reaching a `fret-node` finding. This is not treated as an
  `FNAR-020` blocker; the no-default `fret-node` clippy gate passed.

Review note:

- `FNAR-020` is accepted as DONE_WITH_CONCERNS. The apply/storage seam is complete, but the original
  M1 scope was too broad for one bounded task. The remaining diff/invert/change-projection facade
  work is split to `FNAR-021`.

## FNAR-021 Verification - 2026-05-27

Claim: graph mutation coordination now has a public facade on `GraphTransaction` for diff,
application, inverse generation, and XYFlow-style change projection. Routine production callers no
longer call raw `graph_diff`, `apply_transaction`, `invert_transaction`, or
`NodeGraphChanges::from_transaction` directly.

Evidence anchors:

- `ecosystem/fret-node/src/ops/mod.rs`: `GraphTransaction::{diff,apply_to,inverse}`.
- `ecosystem/fret-node/src/runtime/changes.rs`: `GraphTransaction::node_graph_changes`; raw
  `NodeGraphChanges::from_transaction` is crate-private.
- `ecosystem/fret-node/src/runtime/store.rs` and
  `ecosystem/fret-node/src/ui/canvas/widget/commit/apply.rs`: event publication uses the facade.
- `ecosystem/fret-node/src/ops/tests.rs`: public-seam diff/apply/inverse roundtrip test.

Fresh gates:

- `cargo nextest run -p fret-node graph_transaction_facade_diff_apply_and_inverse_roundtrip`:
  passed. Proves diff/apply/inverse are usable through the public mutation seam.
- `cargo nextest run -p fret-node changes_from_transaction_maps_ops`: passed. Proves basic change
  projection through `GraphTransaction::node_graph_changes`.
- `cargo nextest run -p fret-node changes_to_transaction_is_reversible_and_applicable`: passed.
  Proves change-to-transaction remains applicable after the facade split.
- `cargo nextest run -p fret-node store_dispatch_changes_records_history_and_supports_undo`: passed.
  Proves store dispatch, history, and projected changes remain coherent.
- `cargo nextest run -p fret-node store_dispatch_pipeline_publishes_coherent_commit_state`: passed.
  Proves profile-derived commits still publish coherent committed graph/change state.
- `cargo nextest run -p fret-node --no-default-features`: passed, 246 tests. Proves the headless
  mutation/runtime baseline after the facade tightening.
- `cargo nextest run -p fret-node`: passed, 558 tests. Proves default UI-enabled package behavior.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Gate not accepted as task evidence:

- `cargo clippy -p fret-node --all-targets -- -D warnings`: failed in pre-existing `fret-ui`
  clean-geometry lints (`enum_variant_names`, `large_enum_variant`, `collapsible_if`,
  `redundant_closure_call`) before reaching a `fret-node` finding. This remains outside
  `FNAR-021`; the no-default `fret-node` clippy gate passed.

Review note:

- `FNAR-021` is accepted. Remaining raw `GraphOp` construction is still allowed for advanced
  transaction authorship and internal adapter tests; full-fidelity patch semantics are explicitly
  deferred to `FNAR-050`.

## FNAR-030 Verification - 2026-05-27

Claim: `NodeGraphStore` now owns document replacement as an atomic store operation. Full document
replacement updates graph, view state, editor config, graph revision, lookups, and history in one
store mutation and emits one `DocumentReplaced` event. Graph-only replacement also emits
`DocumentReplaced` while preserving caller-owned view/history policy.

Evidence anchors:

- `ecosystem/fret-node/src/runtime/events.rs`: `NodeGraphDocumentSnapshot` and
  `NodeGraphStoreEvent::DocumentReplaced`.
- `ecosystem/fret-node/src/runtime/store.rs`: `replace_graph`, `replace_document`, graph revision
  snapshots, and atomic document replacement publication.
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`: controller replace-document path calls
  `NodeGraphStore::replace_document` instead of sequencing separate graph/view/config updates.
- `docs/node-graph-controlled-mode.md`: replace policy documents the `DocumentReplaced` event and
  history behavior.

Fresh gates:

- `cargo nextest run -p fret-node store_`: passed, 51 tests. Proves store authority, document
  replacement, revision, selector, and store-first UI paths.
- `cargo nextest run -p fret-node install_callbacks`: passed, 3 tests. Proves the new replacement
  event does not misfire incremental graph/view callbacks.
- `cargo nextest run -p fret-node store_replace_document_emits_single_document_event_and_clears_history`:
  passed. Proves full document replacement emits exactly one document event, sanitizes selection,
  applies editor config, advances revision, and clears history.
- `cargo nextest run -p fret-node store_replace_graph_emits_document_event_and_preserves_history_policy`:
  passed. Proves graph-only replacement emits document replacement while preserving history policy.
- `cargo nextest run -p fret-node controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper public_node_graph_guides_teach_binding_first_surface`:
  passed, 2 tests. Proves controlled-mode docs and binding-first guide remain coherent after the
  public seam update.
- `cargo nextest run -p fret-node --no-default-features`: passed, 248 tests. Proves the headless
  store/event contract.
- `cargo nextest run -p fret-node`: passed, 560 tests. Proves default UI-enabled package behavior.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Review note:

- `FNAR-030` is accepted. Remaining mirror model cleanup is intentionally deferred to
  `FNAR-070`, because current declarative runtime already reads the authoritative store and the
  broad source-text mirror assertions need seam-test replacement before deletion.

## FNAR-040 Verification - 2026-05-27

Claim: editor/project state persistence no longer pretends to be pure view-state persistence.
`GraphFileV1` remains the graph document wrapper, while `NodeGraphEditorStateFile` stores only
per-project editor state: pure `view_state` plus nested `editor_config` (`interaction` policy and
`runtime_tuning`). Historical plain/root and `state_version = 2` loader compatibility paths were
deleted for this fearless-refactor lane.

Evidence anchors:

- `ecosystem/fret-node/src/io/mod.rs`: `EDITOR_STATE_FILE_VERSION`,
  `default_project_editor_state_path`, `NodeGraphEditorStateFile`, and
  `NodeGraphEditorStateFileError`.
- `ecosystem/fret-node/src/io/mod.rs`: editor-state persistence tests for split serialization,
  unsupported versions, graph id mismatch, interaction/runtime split, and stale view-state
  sanitization.
- `docs/adr/0126-node-graph-editor-and-typed-connections.md`: editor-state persistence path and
  locked wrapper shape.
- `docs/node-graph-xyflow-parity.md`: viewport persistence helper names updated to the new seam.

Fresh gates:

- `cargo nextest run -p fret-node editor_state_file`: passed, 3 tests. Proves the new persistence
  helper roundtrips and rejects wrong graph ids / unsupported versions.
- `cargo nextest run -p fret-node io::`: passed, 5 tests. Proves the IO split, interaction/runtime
  separation, and view-state sanitation.
- `cargo nextest run -p fret-node core::`: passed, 13 tests. Proves the semantic graph validation
  suite still passes after persistence split cleanup.
- `cargo nextest run -p fret-node --no-default-features`: passed, 247 tests. Proves the headless
  package after deleting old editor-state migration compatibility.
- `cargo nextest run -p fret-node`: passed, 559 tests. Proves default UI-enabled package behavior.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Review note:

- `FNAR-040` is accepted. Element-level XyFlow flags on nodes, ports, and edges remain document
  annotations rather than session/editor runtime state; per-project editor policy now lives behind
  `NodeGraphEditorConfig` and the new editor-state persistence seam.

## FNAR-050 Verification - 2026-05-27

Claim: full-fidelity graph patches are now the primary runtime commit payload. `NodeGraphPatch`
wraps the committed `GraphTransaction`; `NodeGraphStoreEvent::GraphCommitted` emits
`{ patch, node_edge_changes }`; middleware, dispatch outcomes, store subscriptions, and callback
adapters treat `NodeGraphChanges` as an explicitly lossy XyFlow-style node/edge projection.

Evidence anchors:

- `ecosystem/fret-node/src/runtime/changes.rs`: `NodeGraphPatch` plus `NodeGraphChanges` adapter
  documentation.
- `ecosystem/fret-node/src/runtime/events.rs`: `GraphCommitted { patch, node_edge_changes }`.
- `ecosystem/fret-node/src/runtime/store.rs`: `DispatchOutcome { patch, node_edge_changes }` and
  patch-first publish path.
- `ecosystem/fret-node/src/runtime/callbacks.rs`: `NodeGraphCommitCallbacks::on_graph_commit`
  receives `NodeGraphPatch`, then `on_node_edge_changes` receives the adapter projection.
- `ecosystem/fret-node/src/runtime/tests.rs`: `install_callbacks_receives_full_patch_for_port_only_commits`
  proves port-only commits remain visible in the full patch while node/edge changes are empty.
- `docs/node-graph-controlled-mode.md` and `docs/node-graph-xyflow-parity.md`: controlled-mode
  docs describe patch-first commits and lossy node/edge projection.

Fresh gates:

- `cargo nextest run -p fret-node changes_from_transaction install_callbacks store_dispatch_pipeline_publishes_coherent_commit_state`:
  passed, 10 tests. Proves change projection, callback installation, and coherent commit
  publication.
- `cargo nextest run -p fret-node store_dispatch_pipeline_publishes_coherent_commit_state install_callbacks_receives_full_patch_for_port_only_commits`:
  passed, 2 tests. Proves full patch visibility for a non-node/edge port mutation.
- `cargo nextest run -p fret-node --no-default-features`: passed, 248 tests. Proves the headless
  patch/event contract.
- `cargo nextest run -p fret-node`: passed, 560 tests. Proves default UI-enabled package behavior.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Review note:

- `FNAR-050` is accepted. The public name `NodeGraphChanges` remains for XyFlow parity, but every
  primary runtime seam now names it `node_edge_changes` and carries `NodeGraphPatch` first.

## FNAR-060 Verification - 2026-05-27

Claim: domain-neutral canvas rectangle math and static scene tile planning no longer live only in
`fret-node`. `fret-canvas` now owns reusable rect helpers and cache tile planning helpers, while
`fret-node` delegates through node-graph adapters and keeps node/edge rendering policy local.

Evidence anchors:

- `ecosystem/fret-canvas/src/view/rect.rs`: generic `rect_from_points`, containment, union,
  intersection, and inflation helpers.
- `ecosystem/fret-canvas/src/cache/scene_op_tile_cache.rs`: generic tile rect, tiled-cache
  threshold, next-power-of-two, and centered single-tile planning helpers.
- `ecosystem/fret-node/src/ui/canvas/widget/rect_math_core.rs` and
  `ecosystem/fret-node/src/ui/declarative/paint_only/surface_math.rs`: node graph adapters delegate
  generic rect math to `fret-canvas`.
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan/tiles.rs` and
  `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_key.rs`: node graph cache
  planning delegates generic tile math to `fret-canvas`.
- `ecosystem/fret-node/src/ui/canvas/widget/static_scene_cache_plan.rs`: deleted after its generic
  helpers moved to `fret-canvas`.

Fresh gates:

- `cargo nextest run -p fret-canvas`: passed, 70 tests. Proves extracted generic canvas helpers and
  existing canvas cache/view/spatial behavior.
- `cargo nextest run -p fret-node --features compat-retained-canvas`: passed, 1203 tests. Proves
  retained compatibility canvas paths still compile and run through the node graph adapters.
- `cargo nextest run -p fret-node --no-default-features`: passed, 248 tests. Proves headless
  package behavior after the cross-crate extraction.
- `cargo nextest run -p fret-node`: passed, 560 tests. Proves default UI-enabled package behavior.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed. Proves the new `fret-node` -> `fret-canvas`
  dependency direction is accepted by repository layering policy.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
- `cargo clippy -p fret-canvas --all-targets -- -D warnings`: passed.

Review note:

- `FNAR-060` is accepted. This slice deliberately moved only policy-free geometry/cache planning;
  node graph route preparation, callbacks, and node/edge render semantics remain in `fret-node`.

## FNAR-070 Verification - 2026-05-27

Claim: broad implementation-shape source-text policy tests have been reduced to narrow guardrails,
and the deleted coverage now relies on compile, behavior, transaction, event, and diagnostics
seams. `surface_policy_tests.rs` no longer asserts retained-bridge absence route by route.

Evidence anchors:

- `ecosystem/fret-node/src/surface_policy_tests.rs`: reduced from 5101 lines to 1293 lines;
  removed 108 broad source-policy test blocks and 41 now-unused `include_str!` constants.
- `ecosystem/fret-node/src/surface_policy_tests.rs`:
  `retained_canvas_deleted_compat_facade_stays_out_of_ui_sources` replaces six repeated
  deleted-facade scans with one narrow migration guard.
- Existing behavior seams now carry the contracts that were previously frozen by source text:
  `ecosystem/fret-node/src/runtime/tests.rs`, `ecosystem/fret-node/src/ui/binding.rs`,
  `ecosystem/fret-node/src/ui/controller.rs`, `ecosystem/fret-node/src/ui/canvas/**/tests*.rs`,
  and `ecosystem/fret-node/src/ui/overlays/**/tests*.rs`.

Fresh gates:

- `cargo nextest run -p fret-node --no-default-features retained_canvas_deleted_compat_facade_stays_out_of_ui_sources retained_bridge_source_usage_stays_on_the_migration_ledger`:
  passed, 2 tests. Proves the remaining retained source-text checks are narrow migration guards.
- `cargo nextest run -p fret-node --no-default-features`: passed, 134 tests. Proves headless graph,
  mutation, store, persistence, callback, and narrow source-policy gates after deletion.
- `cargo nextest run -p fret-node`: passed, 446 tests. Proves default UI behavior and compile seams
  after deleting broad source-shape assertions.
- `cargo nextest run -p fret-node --features compat-retained-canvas`: passed, 1089 tests. Proves
  the retained compatibility feature still compiles and behaves without route-by-route source
  assertions.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Review note:

- `FNAR-070` is accepted. Remaining source-policy tests intentionally cover public module shape,
  first-party usage/docs, migration-ledger scans, and crate-private compatibility boundaries; the
  route-level retained bridge checks are now covered by package compile gates and behavior tests.

## FNAR-080 Closeout Verification - 2026-05-27

Claim: the `fret-node` architecture fearless refactor v2 lane is complete. All six target refactor
themes landed and are recorded with fresh evidence.

Closeout gates:

- `cargo fmt --check`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 134 tests.
- `cargo nextest run -p fret-node`: passed, 446 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`: passed, 1089 tests.
- `cargo nextest run -p fret-canvas`: passed, 70 tests.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
- `cargo clippy -p fret-canvas --all-targets -- -D warnings`: passed.

Closeout note:

- No required follow-on is split from this lane. Future optional work can continue by deleting the
  remaining retained compatibility island or expanding generic canvas extraction, but neither is
  needed for the six-refactor objective completed here.
