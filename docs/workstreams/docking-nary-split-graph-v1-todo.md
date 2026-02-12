# Docking N-ary Split Graph — TODO Tracker (v1)

Status: Draft (workstream tracker; normative contracts live in ADRs)

This tracker is intentionally task-first. See the design doc:

- `docs/workstreams/docking-nary-split-graph-v1.md`

## Tracking format

Each TODO is labeled:

- ID: `DN-{priority}-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

## P0 — Core graph correctness and canonical form

- [~] DN-P0-core-001 Define and document canonical-form invariants for N-ary splits.
  - Output: `DockGraph::simplify_*` API surface + doc comments.
  - Gate: unit tests in `crates/fret-core`.
  - Status: canonicalization exists (`simplify_window_forest`), but the “public query/helpers” surface is still evolving.

- [x] DN-P0-core-002 Make `collapse_empty_tabs_upwards` and related helpers N-ary safe.
  - Remove binary-only assumptions (e.g. `children.len() == 2`).
  - Gate: add regression tests that construct 3+ child splits and remove tabs in the middle.
  - Evidence: `crates/fret-core/src/dock/tests.rs` (`close_panel_prunes_empty_tabs_in_nary_split`).

- [x] DN-P0-core-003 Implement “insert instead of wrap” for `DockGraph::move_panel_between_windows`.
  - Insert into nearest same-axis split when possible.
  - Update fractions by splitting the target share (do not reset to 50/50).
  - Gate: unit tests for repeated edge-dock sequences (tree depth does not grow).
  - Evidence: `crates/fret-core/src/dock/tests.rs` (`edge_dock_inserts_into_existing_same_axis_split_and_splits_share`).

- [x] DN-P0-core-004 Implement “insert instead of wrap” for `DockGraph::move_tabs_between_windows`.
  - Gate: unit tests for moving whole tab stacks.

- [~] DN-P0-core-005 Add a post-op simplification pipeline.
  - Steps: prune empty tabs, prune single-child splits, flatten nested same-axis splits, normalize fractions.
  - Gate: “round trip” tests with randomized op sequences (bounded depth).
  - Status: deterministic canonicalization is in place; randomized/fixture op-sequence coverage is still TODO.

- [x] DN-P0-core-006 Update `DockGraph::compute_layout` to avoid silent truncation.
  - Repair non-canonical splits locally (mismatched lengths, non-finite shares) for deterministic layout.
  - Evidence:
    - `crates/fret-core/src/dock/query.rs` (`DockGraph::compute_layout`)
    - `crates/fret-core/src/dock/tests.rs` (`compute_layout_repairs_mismatched_fraction_lengths_without_truncating_children`)

- [x] DN-P0-core-007 Add a `DockGraph` helper to locate a node’s parent chain efficiently.
  - Goal: avoid repeated subtree scans in hot paths (especially for large layouts).
  - Evidence:
    - `crates/fret-core/src/dock/query.rs` (`DockGraph::build_parent_index_for_window`, `DockGraph::edge_dock_decision`)

- [~] DN-P0-core-008 Introduce internal “shares” vocabulary helpers.
  - Example: `normalize_shares(&mut [f32])`, `split_share(old, k) -> (a, b)`.
  - Goal: keep semantics explicit even if persisted field remains `fractions`.
  - Status: `normalize_shares` + share splitting helper exists in `crates/fret-core/src/dock/mutate.rs`; expand as needed when UI clamping lands.

## P0 — UI alignment (drop previews and split handles)

- [x] DN-P0-ui-001 Update docking drop preview geometry to match commit semantics.
  - If we insert into a same-axis split, preview must show the new child slot.
  - Gate: add deterministic geometry tests in `ecosystem/fret-docking`.
  - Status: drop overlay preview consults `DockGraph::edge_dock_decision`.
  - Gate: `ecosystem/fret-docking/src/dock/tests/drop_hints.rs` includes an edge-insert overlay rect test.
  - Evidence: `ecosystem/fret-docking/src/dock/tests/drop_hints.rs` (`dock_edge_drop_overlay_previews_insert_into_same_axis_split_slot`).

- [x] DN-P0-ui-002 Ensure split-handle hit-testing works for N-ary splits.
  - Verify `resizable_panel_group::compute_layout` returns N-1 handles and we respect them.
  - Gate: tests in `ecosystem/fret-docking/src/dock/tests/*`.
  - Evidence: `ecosystem/fret-docking/src/dock/tests/split.rs` (`nary_split_handle_hit_test_reports_correct_handle_index`).

- [x] DN-P0-ui-003 Rework splitter drag update to adjust only adjacent shares.
  - Emit `DockOp::SetSplitFractions` / `Many` updates that preserve other children.
  - Gate: deterministic unit tests for N-ary splits (adjacent-only behavior).
  - Evidence:
    - `crates/fret-ui/src/retained_bridge.rs` (`drag_update_adjacent_fractions`)
    - `ecosystem/fret-docking/src/dock/space.rs` (divider drag uses adjacent-only update)
    - `ecosystem/fret-docking/src/dock/tests/split.rs` (N-ary adjacency tests)

- [x] DN-P0-ui-004 Reduce or remove nested same-axis stabilization once canonical form is enforced.
  - Target: delete or simplify `ecosystem/fret-docking/src/dock/split_stabilize.rs`. (done; removed)
  - Gate: existing splitter tests must remain green.
  - Evidence:
    - `crates/fret-core/src/dock/persistence.rs` (imports simplify to canonical form)
    - `ecosystem/fret-docking/src/dock/space.rs` (split drag commits only the touched split)

## P1 — Constraints hooks (editor feel)

- [x] DN-P1-policy-001 Add docking policy hooks for `min_size` per panel kind.
  - Start with viewport panels to prevent collapsing.
  - Gate: unit tests for clamping; manual demo verification.
  - Evidence:
    - `ecosystem/fret-docking/src/dock/mod.rs` (`DockingPolicy`, default viewport min)
    - `ecosystem/fret-docking/src/dock/services.rs` (`DockingPolicyService`)
    - `ecosystem/fret-docking/src/dock/space.rs` (split drag clamps via `min_px`)
    - `ecosystem/fret-docking/src/dock/tests/split.rs` (`dock_split_handle_drag_respects_panel_min_size_policy`)

- [x] DN-P1-policy-002 Add drop-zone masks (disallow docking on certain edges/targets).
  - Gate: diag script verifying disallowed zone never commits.
  - Evidence:
    - `ecosystem/fret-docking/src/dock/mod.rs` (`DockingPolicy::allow_dock_drop_target`)
    - `ecosystem/fret-docking/src/dock/space.rs` (drop target resolve filters by policy)
    - `ecosystem/fret-docking/src/dock/tests/drag.rs` (`dock_drag_drop_zone_mask_can_disallow_left_hint_rect`)
    - `crates/fret-diag-protocol/src/lib.rs` (`UiPredicateV1::DockDropResolveSourceIs`, `UiPredicateV1::DockDropResolvedIsSome`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (predicate evaluation + tests)
    - `apps/fret-examples/src/docking_arbitration_demo.rs` (demo toggle + policy wiring)
    - `tools/diag-scripts/docking-arbitration-demo-nary-drop-zone-mask-disallow-left-edge.json`
    - `crates/fret-diag/src/lib.rs` (`docking_arbitration_suite_scripts` includes mask script)

- [x] DN-P1-policy-003 Add group locking / “no-drop-target” semantics.
  - Similar to `egui_tiles` behavior overrides; kept in docking layer.
  - Evidence:
    - `ecosystem/fret-docking/src/dock/mod.rs` (`DockingPolicy::allow_panel_drag`, `DockingPolicy::allow_tabs_group_drag`, `DockingPolicy::allow_tear_off`)
    - `ecosystem/fret-docking/src/dock/space.rs` (gates drag start + tear-off requests)
    - `ecosystem/fret-docking/src/dock/tests/drag.rs` (`dock_drag_start_respects_tabs_group_drag_policy`, `dock_drag_tear_off_request_respects_policy`)

## P1 — Observability (required for diag/perf gates)

- [x] DN-P1-obs-001 Add a small, stable dock graph stats snapshot for diagnostics bundles.
  - Example fields: node_count, max_depth, split_count, tabs_count, floating_count.
  - Must be cheap to compute (or cached per frame).
  - Evidence:
    - `crates/fret-runtime/src/interaction_diagnostics.rs` (`DockGraphStatsDiagnostics`)
    - `ecosystem/fret-docking/src/dock/space.rs` (`dock_graph_stats_for_window`, published via `WindowInteractionDiagnosticsStore`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export: `UiDockGraphStatsDiagnosticsV1`)

- [x] DN-P1-obs-002 Expose “preview kind” in diagnostics when hovering a drop target.
  - Example: `wrap_binary` vs `insert_into_split(axis, index)`.
  - Goal: scripts can assert semantics without pixel checks.
  - Evidence:
    - `crates/fret-runtime/src/interaction_diagnostics.rs` (`DockDropPreviewDiagnostics`, `DockDropPreviewKindDiagnostics`)
    - `ecosystem/fret-docking/src/dock/space.rs` (`compute_dock_drop_resolve_diagnostics` sets `preview`)
    - `crates/fret-diag-protocol/src/lib.rs` (`UiPredicateV1::DockDropPreviewKindIs`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (predicate evaluation + bundle export)

## P1 — Diagnostics (`fretboard diag`) gates

- [x] DN-P1-diag-001 Add a diag suite for N-ary split docking invariants.
  - Scripts target: `docking_arbitration_demo`.
  - Must assert: no stuck capture, correct active tab, drop target matches expectation.
  - Evidence:
    - `tools/diag-scripts/docking-arbitration-demo-nary-preview-insert-into-existing-split.json`
    - `tools/diag-scripts/docking-arbitration-demo-nary-repeated-edge-dock-no-deepen.json`
    - `tools/diag-scripts/docking-arbitration-demo-nary-splitter-drag-resizes-viewports.json`
    - `crates/fret-diag/src/lib.rs` (`docking_arbitration_suite_scripts`)
    - `crates/fret-diag-protocol/src/lib.rs` (`UiPredicateV1` docking predicates)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (predicate evaluation)

- [x] DN-P1-diag-002 Add a scripted “repeated edge-dock does not deepen tree” gate.
  - Evidence: bundle includes dock graph stats or a simplified “depth” counter.
  - Evidence:
    - `tools/diag-scripts/docking-arbitration-demo-nary-repeated-edge-dock-no-deepen.json`
    - `crates/fret-diag-protocol/src/lib.rs` (`dock_graph_node_count_le`, `dock_graph_max_split_depth_le`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (predicate evaluation + tests)

- [x] DN-P1-diag-003 Add a scripted splitter drag gate.
  - Evidence: viewport bounds cross a threshold after drag; graph remains canonical.
  - Evidence:
    - `apps/fret-examples/src/docking_arbitration_demo.rs` (`dock-arb-split-handle-viewport` semantics anchor)
    - `tools/diag-scripts/docking-arbitration-demo-nary-splitter-drag-resizes-viewports.json`

- [x] DN-P1-diag-004 Add `meta.required_capabilities` to docking scripts and fail fast on missing support.
  - Goal: prevent “timeouts as failures” in CI; prefer structured capability errors.
  - Evidence:
    - `tools/diag-scripts/docking-arbitration-demo-nary-drop-zone-mask-disallow-left-edge.json` (`meta.required_capabilities`)
    - `tools/diag-scripts/docking-arbitration-demo-nary-preview-insert-into-existing-split.json` (`meta.required_capabilities`)
    - `tools/diag-scripts/docking-arbitration-demo-nary-repeated-edge-dock-no-deepen.json` (`meta.required_capabilities`)
    - `tools/diag-scripts/docking-arbitration-demo-nary-splitter-drag-resizes-viewports.json` (`meta.required_capabilities`)

## P1 — Performance gates

- [x] DN-P1-perf-001 Add a perf probe for repeated splitter drags in a large layout.
  - Compare before/after CPU time, allocations, and layout node visits (where available).
  - Evidence:
    - `tools/diag-scripts/docking-arbitration-demo-nary-splitter-drag-perf-large-layout-steady.json`
    - `apps/fret-examples/src/docking_arbitration_demo.rs` (`FRET_DOCK_ARB_PRESET=large`)
    - `crates/fret-diag/src/perf_seed_policy.rs` (`docking-arbitration-steady`)

- [x] DN-P1-perf-002 Add a perf probe for tab drag hover (drop hint recomputation).
  - Goal: keep pointer-move steady-state under a chosen threshold.
  - Evidence:
    - `tools/diag-scripts/docking-arbitration-demo-nary-tab-drag-hover-perf-large-layout-steady.json`
    - `apps/fret-examples/src/docking_arbitration_demo.rs` (`FRET_DOCK_ARB_DISALLOW_DROP_TARGETS=1`)

## P2 — Persistence and migration (optional in v1)

- [ ] DN-P2-persist-001 Decide whether to bump `DockLayout` version for any semantic changes.
  - If we keep shape identical and only change op semantics, a bump may be unnecessary.

- [ ] DN-P2-persist-002 If persisting constraints/locks becomes required, define a sidecar schema.
  - Preferred: a docking-layer “UI state” file separate from `DockLayout`.
  - Only move into core if it becomes a hard contract.

## P2 — Follow-ups (not required for v1)

- [ ] DN-P2-grid-001 Evaluate a grid container for docking (egui_tiles-style).
  - Likely belongs in `fret-core` as a new node kind only if it becomes a hard requirement.

- [ ] DN-P2-declarative-001 Track retained-bridge exit impact on docking UI authoring.
  - Workstream: `docs/workstreams/retained-bridge-exit-v1.md`
