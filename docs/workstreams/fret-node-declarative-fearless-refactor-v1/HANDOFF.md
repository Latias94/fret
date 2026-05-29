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
declarative click-edge selection, selected-edge paint/diagnostics, and update-anchor planning
contract slices. The current risk is consumer-facing drift where public extension or store surfaces
look authoritative but bypass the store's contracts or imply unimplemented view-policy parity.

## Active Task

- Task ID: FNDX-055.
- Owner: current Codex session.
- Status: DONE.
- Claim: default declarative EdgeWrapper update-anchor planning now resolves selected/focused edge
  source/target anchors from authoritative port centers. Planning respects global
  `edges_reconnectable`, per-edge `Edge.reconnectable`, endpoint-specific source/target overrides,
  missing port centers, and invalid reconnect radii. Surface diagnostics expose planned anchor
  count. Rendering controls and reconnect drag lifecycle remain follow-up work.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/ui/declarative/paint_only/edge_update_anchors.rs` owns deterministic
    selected/focused edge update-anchor planning and reconnectability resolution.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs` invokes anchor planning
    after internals sync.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/semantics.rs` reports planned update-anchor
    count in surface diagnostics.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs` carries
    `edge_reconnect_endpoint_enabled_resolves_global_and_per_edge_overrides`,
    `collect_edge_update_anchor_infos_uses_selected_and_focused_edges_with_port_centers`,
    `collect_edge_update_anchor_infos_respects_endpoint_override_missing_centers_and_radius`, and
    `node_graph_surface_semantics_reports_selected_edges_count`.
  - Fresh gates passed:
    `cargo check -p fret-node --tests`,
    `cargo nextest run -p fret-node edge_reconnect_endpoint_enabled_resolves_global_and_per_edge_overrides collect_edge_update_anchor_infos_uses_selected_and_focused_edges_with_port_centers collect_edge_update_anchor_infos_respects_endpoint_override_missing_centers_and_radius node_graph_surface_semantics_reports_selected_edges_count`,
    `cargo fmt -p fret-node --check`,
    `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`,
    `git diff --check`,
    `python3 tools/check_layering.py`,
    `cargo check -p fret-node --all-features --tests`,
    `cargo check -p fret-node --no-default-features`,
    `cargo clippy -p fret-node --all-targets --all-features -- -D warnings`, and
    `cargo nextest run -p fret-node` (472 tests).
  - Earlier closeout/package gates for FNDX-010 through FNDX-054 remain recorded in
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

## Blockers

- None for FNDX-055.

## Next Recommended Action

- Pick the next view-policy/public-extension slice with a concrete gate. The strongest candidate is
  rendering the planned update anchors as hit-testable controls with anchor-click priority, then
  starting reconnect drags with the existing threshold/cancel policy. `reconnect_on_drop_empty`
  remains a later lifecycle slice.
