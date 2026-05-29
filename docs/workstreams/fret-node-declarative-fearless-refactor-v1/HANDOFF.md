# `fret-node` Fearless Refactor (v1) - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This workstream remains the active lane for making `fret-node` the declarative-first,
controller/binding-first, editor-grade node graph surface for Fret. Recent work closed the retained
canvas mirror cleanup, concrete declarative overlay/add-on parity gates, and now the first
store/view-policy hazard found in the 2026-05-28 `fret-node` architecture audit, the first default
declarative public-extension decision, and the custom edge path spatial, hit-test, anchor, toolbar,
edge-label, custom edge-label renderer, child-bounds interactive edge-label control, default
declarative click-edge selection, selected-edge paint/diagnostics, update-anchor planning,
rendered update-anchor controls, reconnect-drag lifecycle, valid reconnect drop commit/callback
slices, reconnect gesture start/end callback aliases, active reconnect preview wire paint, and the
minimal `reconnect_on_drop_empty` empty-drop event outcome. The current risk is
consumer-facing drift where public extension or store surfaces look authoritative but bypass the
store's contracts or imply unimplemented view-policy parity.

## Active Task

- Task ID: FNDX-061.
- Owner: current Codex session.
- Status: DONE.
- Claim: default declarative EdgeWrapper update-anchor reconnect drops now honor the minimal
  `reconnect_on_drop_empty` mechanism/event contract. Active empty-canvas drops still default to
  `ConnectEndOutcome::NoOp`; when the config is enabled, the same drop emits
  `ConnectEndOutcome::OpenInsertNodePicker` with `target: None`, clears transient reconnect state,
  and does not commit a graph transaction. Concrete picker UI remains a policy follow-up.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/ui/declarative/paint_only/edge_update_anchors.rs` owns deterministic
    selected/focused edge update-anchor planning, hit-test rects, rendered controls, reconnect
    armed/active pointer lifecycle, target-port hit-testing, and accepted reconnect transaction
    dispatch plus reconnect gesture start/end event emission.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs` paints the active reconnect
    preview wire with existing canvas path, Bezier route, preview color, and dash conventions.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_content.rs`,
    `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`, and
    `ecosystem/fret-node/src/ui/declarative/paint_only/semantics.rs` pass reconnect state into
    paint and diagnostics.
  - `ecosystem/fret-node/src/runtime/events.rs`, `ecosystem/fret-node/src/runtime/store.rs`, and
    `ecosystem/fret-node/src/runtime/callbacks.rs` expose transient gesture events, store
    gesture subscriptions, and reconnect-only callback alias fan-out.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/input_handlers.rs` routes surface-level
    move/up/cancel/Escape events through reconnect drop/cleanup and lifecycle-end emission before
    other canvas gestures.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/frame_plan.rs` and
    `ecosystem/fret-node/src/ui/declarative/paint_only/semantics.rs` expose reconnect
    armed/active diagnostics.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs` carries reconnect state
    through frame preparation and marks internals `connecting` only once active.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_content.rs` places update-anchor
    controls in the interactive overlay layer and passes the per-frame drop context needed by
    reconnect hit-testing.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_shell.rs` supplies the same drop
    context to the surface-level pointer-up route so captured pointer releases and anchor-local
    releases use the same commit path.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs` carries
    `edge_reconnect_endpoint_enabled_resolves_global_and_per_edge_overrides`,
    `collect_edge_update_anchor_infos_uses_selected_and_focused_edges_with_port_centers`,
    `collect_edge_update_anchor_infos_respects_endpoint_override_missing_centers_and_radius`,
    `node_graph_surface_semantics_reports_selected_edges_count`,
    `edge_update_anchor_controls_render_and_intercept_before_surface_pointer_down`,
    `edge_update_anchor_controls_respect_endpoint_reconnectable_gate`,
    `edge_update_anchor_drag_uses_connection_threshold_before_active_reconnect`,
    `edge_update_anchor_reconnect_drag_cancel_paths_clear_transient`,
    `edge_update_anchor_reconnect_drop_on_valid_port_commits_store_transaction_and_callbacks`,
    `edge_update_anchor_reconnect_drop_on_non_start_connectable_port_clears_without_commit`,
    `edge_update_anchor_reconnect_drop_on_empty_space_clears_without_commit`, and
    `edge_update_anchor_reconnect_drop_on_empty_space_opens_insert_node_picker_when_enabled`.
  - Fresh gates passed:
    `cargo nextest run -p fret-node edge_update_anchor_reconnect_drop_on_empty_space_clears_without_commit edge_update_anchor_reconnect_drop_on_empty_space_opens_insert_node_picker_when_enabled`,
    `cargo check -p fret-node --tests`,
    `cargo nextest run -p fret-node edge_update_anchor_reconnect_drop_on_empty_space_clears_without_commit edge_update_anchor_reconnect_drop_on_empty_space_opens_insert_node_picker_when_enabled edge_update_anchor_reconnect_active_drag_paints_preview_wire_until_cleanup edge_update_anchor_reconnect_drop_on_valid_port_commits_store_transaction_and_callbacks edge_update_anchor_reconnect_drop_on_non_start_connectable_port_clears_without_commit edge_update_anchor_reconnect_drag_cancel_paths_clear_transient`,
    `cargo fmt --check`,
    `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`,
    `git diff --check`,
    and `python3 tools/check_layering.py`.
  - Earlier closeout/package gates for FNDX-010 through FNDX-056 remain recorded in
    `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Reuse this existing workstream instead of opening a duplicate XYFlow parity lane.
- Treat `docs/workstreams/standalone/fret-node-xyflow-parity.md` as the historical parity execution
  plan and `docs/node-graph-xyflow-parity.md` as the detailed map.
- Treat the current narrow task as a consumer-surface proof: binding-first docs plus a source-policy
  gate.
- Keep diff-first controlled sync out of the public helper surface for now; require workload
  evidence before adding a `replace_*_with_diff` API.
- Treat FNDX-030 as policy-placement closure, not a full declarative parity claim: remaining
  overlay behavior parity should be split into future focused conformance tasks.
- Keep this workstream active after closeout verification and split concrete overlay behavior gates
  instead of marking the whole lane complete.
- FNDX-040 chose input transparency as the first concrete declarative overlay parity gate and did
  not widen the overlay policy surface.
- FNDX-041 chose motion anchoring as the second concrete declarative overlay parity gate and kept
  the behavior on existing hover-anchor and overlay-spec seams.
- FNDX-042 chose declarative portal text cancel focus return as the next concrete add-on behavior
  gate and kept the implementation on the existing portal command/editor seams.
- FNDX-043 promoted the existing mounted declarative rename overlay Escape/focus-return gate into
  the parity/evidence map instead of duplicating an equivalent test.
- FNDX-044 chose the smallest store/view-policy refactor from the 2026-05-28 audit: delete the
  public raw `view_state_mut` bypass first, then leave broader presenter/skin/edge-type wiring for
  a follow-up slice.
- FNDX-045 wires `NodeGraphEdgeTypes` and `NodeGraphSkin` into the default declarative edge paint
  path, but does not expose custom `NodeGraphPresenter` as a default prop because that trait still
  mixes geometry, labels, context menus, search/insert policy, and rendering hints.
- FNDX-046 extends the custom edge path contract from paint/culling into conservative spatial-index
  candidate rects, but does not claim exact curve/path distance hit-testing.
- FNDX-047 extends the custom edge path contract from conservative spatial candidates into exact
  path-distance filtering, but does not claim edge label placement or EdgeToolbar internals parity.
- FNDX-048 extends the custom edge path contract from hit filtering into default edge-center
  anchors, but does not claim full EdgeLabelRenderer-style child labels or EdgeToolbar composition
  internals parity.
- FNDX-049 extends the custom edge path contract from edge-center internals into declarative
  EdgeToolbar composition, but does not claim full EdgeLabelRenderer-style child labels.
- FNDX-050 extends the custom edge path contract from toolbar composition into default visible
  `EdgeRenderHint.label` child-layer placement, but does not claim arbitrary EdgeLabelRenderer-style
  custom child renderers.
- FNDX-051 extends the edge-label contract into non-interactive custom child renderer placement.
- FNDX-052 extends the edge-label contract into opt-in child-bounds pointer-interactive controls,
  but does not claim broader XyFlow EdgeWrapper lifecycle parity or a broad `NodeGraphPresenter`
  public surface.
- FNDX-053 feeds custom-path-aware edge hit-testing into default declarative click-edge selection,
  but does not claim reconnect/update-anchor lifecycle parity.
- FNDX-054 feeds store-backed selected-edge state into default declarative edge paint and
  diagnostics, but still does not claim reconnect/update-anchor lifecycle parity.
- FNDX-055 adds default declarative selected/focused edge update-anchor planning and diagnostics,
  but still does not render anchors or start reconnect drags.
- FNDX-056 renders those planned update anchors as default declarative hit-testable controls with
  anchor-click priority, but still does not start reconnect drags, dispatch reconnect callbacks, or
  implement `reconnect_on_drop_empty`.
- FNDX-057 starts reconnect drags from those rendered update anchors, reuses the existing
  connection-drag threshold and cancel/up cleanup policy, and exposes armed/active diagnostics, but
  still does not target-hit-test, commit reconnect transactions, dispatch reconnect callbacks, paint
  a preview wire, or implement `reconnect_on_drop_empty`.
- FNDX-058 adds target-port hit-testing and accepted reconnect commit/callback dispatch for active
  default declarative update-anchor drags, while keeping empty-canvas drops as cleanup-only and
  respecting endpoint-specific `connectable_start` / `connectable_end` gates; it still defers
  preview wire paint, reconnect gesture start/end callbacks, and `reconnect_on_drop_empty`.
- FNDX-059 emits default declarative reconnect gesture start/end callback aliases for successful
  arm and all current end paths: committed drop, rejected endpoint-gated drop, empty/no-op drop,
  Escape, PointerCancel, and missed-left-button cleanup. It still defers preview wire paint and
  `reconnect_on_drop_empty`.
- FNDX-060 paints an active reconnect preview wire from the fixed port to the current pointer and
  removes it on current cleanup paths.
- FNDX-061 maps opt-in empty-canvas reconnect drops to the insert-node-picker end outcome while
  keeping default empty drops no-op and leaving concrete picker UI as policy follow-up.

## Blockers

- None for FNDX-061.

## Next Recommended Action

- Review the closed reconnect/update-anchor slice, then pick a narrow follow-up only if needed. The
  next strongest candidate is concrete insert-node picker UI/policy for the opt-in empty-drop
  outcome, kept separate from the mechanism-layer reconnect event contract.
