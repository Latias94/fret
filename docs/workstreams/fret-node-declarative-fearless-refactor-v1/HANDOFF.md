# `fret-node` Fearless Refactor (v1) - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This workstream remains the active lane for making `fret-node` the declarative-first,
controller/binding-first, editor-grade node graph surface for Fret. Recent work closed the retained
canvas mirror cleanup, concrete declarative overlay/add-on parity gates, and now the first
store/view-policy hazard found in the 2026-05-28 `fret-node` architecture audit, the first default
declarative public-extension decision, and the custom edge path spatial, hit-test, anchor, toolbar,
edge-label, custom edge-label renderer, and child-bounds interactive edge-label control contract
slices. The current risk is consumer-facing drift where public extension or store surfaces look
authoritative but bypass the store's contracts or imply unimplemented view-policy parity.

## Active Task

- Task ID: FNDX-052.
- Owner: current Codex session.
- Status: DONE.
- Claim: custom `NodeGraphEdgeTypes::register_path(...)` output now feeds default declarative
  `edge_centers_window` anchors, declarative EdgeToolbar host child placement, default
  `EdgeRenderHint.label` child-layer placement, custom edge-label renderer placement, and the first
  opt-in pointer-interactive edge-label control contract through the same custom path midpoint. The
  new `NodeGraphEdgeLabelHitTestMode::ChildBounds` mode limits hit-testing to the measured custom
  child rect; default labels/renderers remain transparent, and points outside an interactive child
  still fall through to the canvas.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/ui/declarative/paint_only/edge_path_geometry.rs` computes
    midpoint/normal anchors from `PathCommand` streams.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs` populates
    `NodeGraphInternalsSnapshot.edge_centers_window` from edge draw commands instead of rebuilding
    the default cubic route center.
  - `ecosystem/fret-node/src/ui/overlays/mod.rs` exposes a test-only internal bridge that resolves
    the declarative EdgeToolbar target from view state + internals and then calls the real
    declarative EdgeToolbar host.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/edge_labels.rs` builds hit-test-transparent
    managed child layers for visible `EdgeRenderHint.label` output and custom renderer children
    from the same internals anchor, with opt-in child-bounds hit-test rects for interactive custom
    children.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/input_handlers.rs` lets descendant
    pressables bypass the graph surface's capture-phase `PointerRegion` handler so custom
    edge-label controls can receive pointer events.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_content.rs` mounts the edge-label
    layer before other overlay children in the default declarative surface.
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` defines
    `NodeGraphDeclarativeEdgeLabelRenderer`, `NodeGraphEdgeLabelLayout`,
    `NodeGraphEdgeLabelHitTestMode`, and `node_graph_surface_with_edge_label_renderer(...)`, plus
    `NodeGraphDeclarativeSurfaceRenderers` / `node_graph_surface_with_renderers(...)`.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs` carries
    `custom_edge_path_feeds_default_declarative_edge_center_anchor` and
    `custom_edge_path_feeds_declarative_edge_toolbar_composition_anchor`, plus
    `custom_edge_path_feeds_declarative_edge_label_child_layer_anchor` and
    `custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor`, plus
    `custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds`.
  - `ecosystem/fret-node/src/surface_policy_tests.rs` now carries
    `default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`.
  - Fresh gates passed:
    `cargo nextest run -p fret-node custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds`
    and
    `cargo nextest run -p fret-node custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor custom_edge_path_feeds_declarative_edge_label_child_layer_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`,
    `cargo check -p fret-node --tests`,
    `cargo check -p fret-node --all-features --tests`,
    `cargo check -p fret-node --no-default-features`,
    `cargo clippy -p fret-node --all-targets --all-features -- -D warnings`,
    `cargo fmt --check`,
    `python3 tools/check_layering.py`,
    `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`,
    `git diff --check`, and
    `cargo nextest run -p fret-node` (464 tests).
  - Earlier closeout/package gates for FNDX-010 through FNDX-050 remain recorded in
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

## Blockers

- None for FNDX-052.

## Next Recommended Action

- Pick the next view-policy/public-extension slice with a concrete gate. The strongest candidate is
  now either broader EdgeWrapper lifecycle parity or the broad `NodeGraphPresenter` split into
  narrower label, geometry, menu, and insertion/search contracts.
