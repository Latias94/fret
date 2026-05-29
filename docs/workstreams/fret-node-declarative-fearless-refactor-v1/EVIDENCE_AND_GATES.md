# `fret-node` Fearless Refactor (v1) - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Current Focus

FNDX-055 starts default declarative EdgeWrapper reconnect/update-anchor parity at the planning
layer. Selected and focused edges now resolve source/target update anchors from authoritative port
centers, with global `edges_reconnectable`, per-edge `Edge.reconnectable`, endpoint override, and
`reconnect_radius` gates. Surface diagnostics expose the planned update-anchor count. This slice
does not render the anchors or start reconnect drags yet.

## Targeted Iteration Gates

```bash
cargo nextest run -p fret-node edge_reconnect_endpoint_enabled_resolves_global_and_per_edge_overrides collect_edge_update_anchor_infos_uses_selected_and_focused_edges_with_port_centers collect_edge_update_anchor_infos_respects_endpoint_override_missing_centers_and_radius node_graph_surface_semantics_reports_selected_edges_count
```

This gate proves the FNDX-055 update-anchor planning contract: global and per-edge reconnectable
settings resolve like XyFlow's `edgesReconnectable` / `edge.reconnectable`, selected and focused
edges produce deterministic source/target endpoint anchors from port centers, missing centers and
invalid reconnect radii suppress anchors, and surface diagnostics report the planned anchor count.

```bash
cargo nextest run -p fret-node edge_stroke_width_mul_for_selection_applies_selected_edge_width_token node_graph_surface_semantics_reports_selected_edges_count custom_edge_path_click_selects_edge_via_default_declarative_pointer_down_path
```

This gate proves the FNDX-054 selected-edge paint/diagnostics contract: default edge paint can derive
selected stroke width from the store-backed selected edge list, surface semantics report selected
edge count for diagnostics, and the FNDX-053 custom-path click-edge selection path still commits the
edge that will feed that selected paint state.

```bash
cargo nextest run -p fret-node custom_edge_path_click_selects_edge_via_default_declarative_pointer_down_path build_click_selection_preview_edges_multi_click_toggles_hit_membership commit_edge_click_selection_action_host_multi_toggles_edge_without_clearing_other_kinds custom_edge_path_hit_testing_uses_exact_path_distance_after_spatial_candidate custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds
```

This gate proves the FNDX-053 edge-click selection contract: the default declarative pointer-down
path receives `edgeTypes`/style policy, uses the custom-path-aware edge hit-test after node hits,
commits single-click edge selection through the store-backed view-state helper, preserves
multi-selection toggle semantics for other selected element kinds, and keeps pointer-interactive
edge-label controls isolated from canvas hit-testing.

```bash
cargo nextest run -p fret-node custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor custom_edge_path_feeds_declarative_edge_label_child_layer_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-052 pointer-interactive edge label control contract: app-provided
edge-label controls can opt into `NodeGraphEdgeLabelHitTestMode::ChildBounds`, reuse the
custom-path-derived edge-center anchor, intercept pointer input only inside the measured child
bounds, let outside points fall through to the canvas, and keep `NodeGraphPresenter` out of the
default declarative surface.

```bash
cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor custom_edge_path_feeds_declarative_edge_label_child_layer_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-051 custom edge label renderer contract: app-provided edge-label children
can render without a default `EdgeRenderHint.label`, receive `NodeGraphEdgeLabelLayout` with the
custom-path-derived edge-center anchor, layout at that anchor, and source-policy/docs keep
`NodeGraphPresenter` out of the default surface while naming pointer-interactive label controls as
follow-up work.

```bash
cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_label_child_layer_anchor custom_edge_path_feeds_declarative_edge_toolbar_composition_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-050 custom edge path label contract: default declarative internals expose
`edge_centers_window` using the custom path midpoint, the declarative edge-label child layer consumes
that anchor for visible `EdgeRenderHint.label` placement, the label host remains hit-test
transparent, and source-policy/docs keep pointer-interactive edge label controls deferred.

```bash
cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_toolbar_composition_anchor custom_edge_path_feeds_default_declarative_edge_center_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-049 custom edge path toolbar contract: default declarative internals expose
`edge_centers_window` using the custom path midpoint, the declarative EdgeToolbar host consumes that
anchor for child placement, and source-policy/docs keep the scoped EdgeLabelRenderer demotion
explicit.

```bash
cargo nextest run -p fret-node custom_edge_path_feeds_default_declarative_edge_center_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-048 custom edge path anchor contract: default declarative internals expose
`edge_centers_window` using the custom path midpoint instead of the default presenter route center,
and source-policy/docs keep the scoped EdgeLabelRenderer/EdgeToolbar demotion explicit.

```bash
cargo nextest run -p fret-node derived_geometry_cache_key_changes_when_edge_types_revision_changes custom_edge_path_spatial_rect_overrides_feed_edge_index_candidates custom_edge_path_hit_testing_uses_exact_path_distance_after_spatial_candidate default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-047 custom edge path hit-testing contract: `edgeTypes` revisions still
invalidate derived geometry, custom edge paths still provide conservative spatial-index edge
candidates, exact path-distance filtering accepts points on the custom path and rejects coarse-AABB
misses, and source-policy/docs keep the scoped edge-label/toolbar demotion explicit.

```bash
cargo nextest run -p fret-node derived_geometry_cache_key_changes_when_edge_types_revision_changes custom_edge_path_spatial_rect_overrides_feed_edge_index_candidates default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-046 custom edge path spatial contract: `edgeTypes` revisions invalidate
derived geometry, custom edge paths provide conservative spatial-index edge candidates, and the
source-policy/docs keep the scoped spatial claim explicit.

```bash
cargo nextest run -p fret-node edges_cache_key_changes_when_edge_types_or_skin_revision_changes declarative_edge_types_feed_default_surface_edge_draws declarative_skin_refines_edge_draw_hints_after_edge_types default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter
```

This gate proves the FNDX-045 default declarative extension decision: edge paint cache invalidation
observes `edgeTypes`/skin revisions, `edgeTypes` feeds draw hints/custom paint paths, skin refines
edge hints after `edgeTypes`, and the source-policy docs keep custom `NodeGraphPresenter` out of the
default surface.

```bash
cargo nextest run -p fret-node store_public_surface_does_not_expose_raw_view_state_mutation
```

This gate proves `NodeGraphStore` does not expose a public raw mutable view-state reference and
keeps public view-state mutation on the notifying/sanitizing helper paths.

```bash
cargo nextest run -p fret-node public_node_graph_guides_teach_binding_first_surface
```

This gate proves the public crate README and the XyFlow-style guide keep the binding-first teaching
surface and do not drift back to direct retained canvas authoring or stale graph/view/model triplets.

```bash
cargo nextest run -p fret-node controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper
```

This gate proves the controlled-mode docs keep the FNDX-020 decision explicit and the public
binding/controller sync surfaces have not grown a hidden diff-first replacement helper.

```bash
cargo nextest run -p fret-node controlled_graph_can_apply_store_changes_via_callbacks
```

This gate proves the controlled runtime path still supports app-owned graph state by applying store
`NodeChange` / `EdgeChange` callbacks with `apply_*_changes`.

```bash
cargo nextest run -p fret-node overlay_menu_toolbar_policy_ownership_stays_on_named_seams
```

This gate proves the FNDX-030 placement decision: toolbar public policy types stay on the toolbar
policy seam, and declarative toolbar composition consumes that seam instead of owning it inline.

```bash
cargo nextest run -p fret-node retained_compatibility_surface_is_removed
```

This gate proves the old retained compatibility surface stays deleted from the current public
surface instead of being revived under a new name.

```bash
cargo nextest run -p fret-node --no-default-features runtime
```

This gate protects the headless runtime/change/store behavior while consumer docs reference
`NodeGraphStore`, controlled mode, and transaction-backed changes.

```bash
cargo nextest run -p fret-node declarative_overlay_layer_is_input_transparent_over_canvas_region
```

This gate proves declarative overlay layers stay hit-test transparent over the canvas region, so
diagnostics-only hover/marquee overlays do not steal pointer input from the underlying surface.

```bash
cargo nextest run -p fret-node declarative_hover_tooltip_overlay_tracks_dragged_anchor_when_portals_disabled
```

This gate proves diagnostics hover-tooltip overlay placement follows drag-adjusted hover anchors
when portal bounds are disabled or unavailable.

```bash
cargo nextest run -p fret-node declarative_portal_text_cancel_returns_focus_to_surface_without_graph_commit
```

This gate proves declarative portal text cancel commands are available for live portal nodes, return
focus to the graph surface, and do not commit graph/store changes.

```bash
cargo nextest run -p fret-node rename_managed_host_escape_closes_without_transaction_and_restores_focus
```

This gate proves a mounted declarative rename overlay text-input subtree closes on Escape without a
graph transaction and restores focus to the graph surface target.

## Package And Boundary Gates

```bash
cargo check -p fret-node --no-default-features
cargo check -p fret-node --all-features --tests
cargo clippy -p fret-node --all-targets --all-features -- -D warnings
python3 tools/check_layering.py
```

Use the no-default-features check when changing headless/runtime docs or exports. Use the
all-features test-target check when touching UI-enabled or optional integration boundaries. Use
the clippy gate when changing public API, traits, or cross-crate integration surfaces. Use layering
checks when moving mechanisms across `fret-node`, `fret-canvas`, or core crates.

## Closeout Gates

```bash
cargo fmt --check
cargo nextest run -p fret-node
cargo check -p fret-node --all-features --tests
```

Closeout should use narrower gates only when the workspace is blocked by unrelated failures, and the
closeout note must name those failures.

## Evidence Anchors

- `docs/node-graph-how-to-build-like-xyflow.md`
- `docs/node-graph-xyflow-parity.md`
- `docs/node-graph-controlled-mode.md`
- `ecosystem/fret-node/README.md`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/ui/edge_types.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/edge_hit_test.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/edge_labels.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/edge_path_geometry.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/edge_update_anchors.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/semantics.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/input_handlers.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/pointer_down.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/selection.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_content.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_shell.rs`
- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbar_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/hover_anchor.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/overlay_elements.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/transactions.rs`
- `ecosystem/fret-node/src/ui/portal_commands.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_command.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/design.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/todo.md`

## Historical Evidence - 2026-05-27

- `cargo nextest run -p fret-node controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper`: passed; proves controlled sync docs and public binding/controller sync sources stay full-replace-first and do not expose diff-first helpers.
- `cargo nextest run -p fret-node controlled_graph_can_apply_store_changes_via_callbacks`: passed; proves the current controlled callback/apply path still mirrors store changes into app-owned graph state.
- `cargo fmt --check`: passed; proves the Rust formatting gate is clean after the new source-policy test.
- Broader package/closeout gates were not rerun for FNDX-020 because this slice only changed docs,
  a source-policy test, and workstream notes; use the package/closeout gate list above before
  accepting broader lane closure.
- `cargo nextest run -p fret-node --features compat-retained-canvas overlay_menu_toolbar_policy_ownership_stays_on_named_seams`: passed; proves the FNDX-030 ownership gate for toolbar public policy, menu/searcher policy enums, and retained menu/searcher lifecycle seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas overlay_policy_modules_compile_without_retained_canvas_compat default_overlay_policy_surfaces_stay_off_retained_bridge`: passed; proves adjacent overlay policy gates still pass with the retained compatibility feature enabled.
- `cargo fmt --check`: passed after the FNDX-030 source-policy test was formatted.
- Broader package/closeout gates were not rerun for FNDX-030 because this slice only adds a
  source-policy gate and workstream notes; use the package/closeout gate list above before
  accepting broader lane closure.
- Review/verify follow-up for FNDX-010/FNDX-020/FNDX-030:
  - `cargo check -p fret-node --no-default-features`: passed; proves the headless/runtime-facing
    package still compiles without default features after the public guide and controlled-mode
    policy updates.
  - `cargo check -p fret-node --features compat-retained-canvas`: passed; proves the retained
    compatibility package surface still compiles after the overlay/menu/toolbar policy ownership
    gate.
  - `python3 tools/check_layering.py`: passed; proves the completed FNDX slices did not violate
    workspace layering policy.
- Closeout verification for FNDX-010/FNDX-020/FNDX-030:
  - `cargo fmt --check`: passed; proves the workspace formatting gate is clean after the FNDX
    slices.
  - `cargo nextest run -p fret-node`: passed; proves the package test suite remains green after the
    public guide, controlled-mode policy, and overlay/menu/toolbar source-policy slices.
  - `cargo check -p fret-node --features compat-retained-canvas --tests`: passed; proves retained
    compatibility test targets still compile after the overlay policy placement closure.
- FNDX-040:
  - `cargo nextest run -p fret-node declarative_overlay_layer_is_input_transparent_over_canvas_region`:
    passed; proves the declarative overlay layer remains input-transparent even if an overlay child
    contains a pointer region.
  - `cargo check -p fret-node --features compat-retained-canvas --tests`: passed; proves retained
    compatibility test targets still compile with the new declarative overlay behavior gate.
  - `cargo fmt --check`: passed; proves formatting is clean after the new Rust test.
- FNDX-041:
  - `cargo nextest run -p fret-node declarative_hover_tooltip_overlay_tracks_dragged_anchor_when_portals_disabled`:
    passed; proves the final diagnostics hover-tooltip overlay spec tracks drag-adjusted hover
    anchors when portal bounds are disabled.
  - `cargo check -p fret-node --features compat-retained-canvas --tests`: passed; proves retained
    compatibility test targets still compile with the new motion-anchoring gate.
  - `cargo fmt --check`: passed; proves formatting is clean after the new Rust test.
- Review/package follow-up after FNDX-040/FNDX-041:
  - `cargo nextest run -p fret-node`: passed; proves the full package test suite remains green with
    the new declarative overlay input-transparency and motion-anchoring gates.
- FNDX-042:
  - `cargo nextest run -p fret-node declarative_portal_text_cancel_returns_focus_to_surface_without_graph_commit`:
    passed; proves a declarative portal text add-on cancel command is available for live portal
    nodes, returns focus to the graph surface, and does not commit graph/store changes.
  - `cargo check -p fret-node --features compat-retained-canvas --tests`: passed; proves retained
    compatibility test targets still compile with the new portal focus-return gate.
  - `cargo fmt --check`: passed; proves formatting is clean after the new Rust test.
- Review/package follow-up after FNDX-042:
  - `cargo nextest run -p fret-node`: passed; proves the full package test suite remains green with
    the new declarative portal text cancel focus-return gate.
- FNDX-043:
  - `cargo nextest run -p fret-node rename_managed_host_escape_closes_without_transaction_and_restores_focus`:
    passed; proves a mounted declarative rename overlay text-input subtree closes on Escape without
    a graph transaction and restores focus to the graph surface target.
- Review/package follow-up after FNDX-043:
  - `cargo nextest run -p fret-node`: passed; proves the full package test suite remains green with
    the mounted declarative rename overlay dismissal/focus-return parity gate.

## Fresh Evidence - 2026-05-28

- FNDX-044:
  - `cargo nextest run -p fret-node store_public_surface_does_not_expose_raw_view_state_mutation`:
    passed; proves `NodeGraphStore` no longer exposes public raw mutable view-state access and
    keeps writes on notifying/sanitizing helper paths.
  - `cargo fmt --check`: passed; proves formatting is clean after the source-policy and workstream
    updates.
  - `cargo check -p fret-node --no-default-features`: passed; proves the headless/runtime-facing
    package still compiles after the store API removal.
  - `cargo nextest run -p fret-node --no-default-features runtime`: passed; proves headless runtime
    store/change behavior remains green without default UI features.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets still compile after the public store surface change.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 452 tests.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
- FNDX-045:
  - `cargo fmt --check`: passed; proves formatting is clean after the Rust and workstream updates.
  - `cargo check -p fret-node --tests`: passed; proves the UI-enabled test targets compile after
    wiring `NodeGraphSurfaceProps.edge_types` / `NodeGraphSurfaceProps.skin` into the declarative
    frame/cache path.
  - `cargo nextest run -p fret-node edges_cache_key_changes_when_edge_types_or_skin_revision_changes declarative_edge_types_feed_default_surface_edge_draws declarative_skin_refines_edge_draw_hints_after_edge_types default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves edge paint caches key on `edgeTypes`/skin revisions, `edgeTypes` supplies default
    declarative edge draw hints/custom paint paths, skin refines edge hints after `edgeTypes`, and
    source-policy/docs keep custom `NodeGraphPresenter` deferred from the default surface.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets still compile with the new default-surface extension props.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing package
    compilation remains unaffected by the UI-only extension slice.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 456 tests.
- FNDX-046:
  - `cargo fmt --check`: passed; proves formatting is clean after the Rust and workstream updates.
  - `cargo check -p fret-node --tests`: passed; proves the UI-enabled test targets compile after
    wiring `edgeTypes` custom path spatial rect overrides into the derived cache path.
  - `cargo nextest run -p fret-node derived_geometry_cache_key_changes_when_edge_types_revision_changes custom_edge_path_spatial_rect_overrides_feed_edge_index_candidates default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves `edgeTypes` revisions invalidate derived geometry, custom path conservative
    AABBs feed the edge spatial candidate set, and source-policy/docs keep the scoped spatial
    contract explicit.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets still compile with custom-path spatial candidate wiring.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing package
    compilation remains unaffected by the UI-only spatial candidate slice.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 458 tests.
- FNDX-047:
  - `cargo fmt --check`: passed; proves formatting is clean after the new edge hit-test helper and
    workstream updates.
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile after the
    custom-path exact hit filtering helper and spatial padding update.
  - `cargo nextest run -p fret-node custom_edge_path_hit_testing_uses_exact_path_distance_after_spatial_candidate`:
    passed; proves points on the custom path hit the edge while points inside the conservative
    custom-path AABB but outside the interaction-width path distance are rejected.
  - `cargo nextest run -p fret-node derived_geometry_cache_key_changes_when_edge_types_revision_changes custom_edge_path_spatial_rect_overrides_feed_edge_index_candidates custom_edge_path_hit_testing_uses_exact_path_distance_after_spatial_candidate default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves FNDX-046 spatial candidates, FNDX-047 exact hit filtering, derived invalidation,
    and source-policy/docs remain aligned.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets still compile with custom-path hit filtering.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing package
    compilation remains unaffected by the UI-only edge hit-test slice.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 459 tests.
- FNDX-048:
  - `cargo nextest run -p fret-node custom_edge_path_feeds_default_declarative_edge_center_anchor`:
    passed; proves default declarative internals use the custom path midpoint for
    `edge_centers_window` instead of the default cubic route midpoint.
  - `cargo nextest run -p fret-node custom_edge_path_feeds_default_declarative_edge_center_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves the custom path anchor behavior and source-policy/docs are aligned while full
    EdgeLabelRenderer-style labels and EdgeToolbar composition internals remain scoped follow-ups.
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with the
    new edge path geometry helper.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with the custom path anchor helper.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing package
    compilation remains unaffected by the UI-only anchor slice.
  - `cargo fmt --check`: passed; proves formatting remains clean.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 460 tests.
- FNDX-049:
  - `cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_toolbar_composition_anchor`:
    passed; proves declarative EdgeToolbar host child placement consumes the custom-path-derived
    edge-center internals produced by the default surface.
  - `cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_toolbar_composition_anchor custom_edge_path_feeds_default_declarative_edge_center_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves the toolbar composition gate, the FNDX-048 edge-center anchor gate, and the
    source-policy/docs demotion of full EdgeLabelRenderer-style labels stay aligned.
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with the
    test-only EdgeToolbar internals bridge.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with the new toolbar composition gate.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing package
    compilation remains unaffected by the UI-only toolbar composition slice.
  - `cargo fmt --check`: passed; proves formatting remains clean.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 461 tests.
- FNDX-050:
  - `cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_label_child_layer_anchor`:
    passed; proves default `EdgeRenderHint.label` output renders through a declarative edge-label
    child layer centered on the custom-path-derived edge-center internals.
  - `cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_label_child_layer_anchor custom_edge_path_feeds_declarative_edge_toolbar_composition_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves the label child-layer gate, toolbar composition gate, and source-policy/docs
    demotion of pointer-interactive edge label controls stay aligned.
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with the new
    edge-label overlay child layer.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with the new edge-label child-layer gate.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing package
    compilation remains unaffected by the UI-only edge-label child-layer slice.
  - `cargo fmt --check`: passed; proves formatting remains clean.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with the new edge-label child-layer gate.
- FNDX-051:
  - `cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor`:
    passed; proves an app-provided edge-label renderer can render a custom child at the
    custom-path-derived edge-center anchor even when `EdgeRenderHint.label` is absent.
  - `cargo nextest run -p fret-node custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor custom_edge_path_feeds_declarative_edge_label_child_layer_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves the custom renderer gate, default label gate, and source-policy/docs stay aligned
    without exposing broad `NodeGraphPresenter` on the default surface.
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with the new
    edge-label renderer entrypoint and re-exports.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with the renderer contract.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing package
    compilation remains unaffected by the UI-only renderer slice.
  - `cargo clippy -p fret-node --all-targets --all-features -- -D warnings`: passed; proves the
    new public renderer trait/helper surface and its UI dependency path are lint-clean.
  - `cargo fmt --check`: passed; proves formatting remains clean.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with the new custom edge-label renderer gate.
- FNDX-052:
  - `cargo nextest run -p fret-node custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds`:
    passed; proves an app-provided edge-label control can opt into child-bounds hit-testing, render
    at the custom-path-derived edge-center anchor, intercept pointer down inside its measured child
    rect, and let points outside that rect fall through.
  - `cargo nextest run -p fret-node custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor custom_edge_path_feeds_declarative_edge_label_child_layer_anchor default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`:
    passed; proves the new interactive control gate, prior custom renderer/default label gates, and
    source-policy/docs stay aligned while `NodeGraphPresenter` remains deferred from the default
    surface.
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with the
    new edge-label hit-test mode and surface descendant-pressable bypass.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with the new public hit-test mode.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing
    package compilation remains unaffected by the UI-only edge-label control slice.
  - `cargo clippy -p fret-node --all-targets --all-features -- -D warnings`: passed; proves the
    public enum/trait-method addition and default-surface event path are lint-clean.
  - `cargo fmt --check`: passed; proves formatting remains clean.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 464 tests.
- FNDX-053:
  - `cargo nextest run -p fret-node custom_edge_path_click_selects_edge_via_default_declarative_pointer_down_path build_click_selection_preview_edges_multi_click_toggles_hit_membership commit_edge_click_selection_action_host_multi_toggles_edge_without_clearing_other_kinds custom_edge_path_hit_testing_uses_exact_path_distance_after_spatial_candidate custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds`:
    passed; proves default declarative pointer-down consumes custom-path-aware edge hit-testing,
    click-edge selection commits through the store-backed view-state path, multi edge-click toggles
    preserve other element kinds, exact custom-path filtering still rejects misses, and
    pointer-interactive edge-label controls remain isolated.
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with the
    new pointer-down edge hit path.
  - `cargo fmt -p fret-node --check`: passed; proves the touched crate remains formatted without
    running a full-workspace formatter.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with the new input path.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing
    package compilation remains unaffected by the UI-only edge-click selection slice.
  - `cargo clippy -p fret-node --all-targets --all-features -- -D warnings`: passed; proves the
    input-path and selection-helper changes are lint-clean.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 467 tests.
- FNDX-054:
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with
    selected-edge paint threading and diagnostics updates.
  - `cargo nextest run -p fret-node edge_stroke_width_mul_for_selection_applies_selected_edge_width_token node_graph_surface_semantics_reports_selected_edges_count custom_edge_path_click_selects_edge_via_default_declarative_pointer_down_path`:
    passed; proves selected-edge paint applies the selected wire-width token, surface diagnostics
    expose selected edge count, and the custom-path click-edge selection path still feeds the store
    state used by paint.
  - `cargo fmt -p fret-node --check`: passed; proves the touched crate remains formatted.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with selected-edge paint threading.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing
    package compilation remains unaffected by the UI-only paint slice.
  - `cargo clippy -p fret-node --all-targets --all-features -- -D warnings`: passed; proves the
    paint/semantics/test changes are lint-clean.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 469 tests.
- FNDX-055:
  - `cargo check -p fret-node --tests`: passed; proves UI-enabled test targets compile with
    update-anchor planning and diagnostics wiring.
  - `cargo nextest run -p fret-node edge_reconnect_endpoint_enabled_resolves_global_and_per_edge_overrides collect_edge_update_anchor_infos_uses_selected_and_focused_edges_with_port_centers collect_edge_update_anchor_infos_respects_endpoint_override_missing_centers_and_radius node_graph_surface_semantics_reports_selected_edges_count`:
    passed; proves global/per-edge reconnectable resolution, selected/focused edge anchor
    collection, endpoint override/missing-center/radius suppression, and diagnostics anchor count.
  - `cargo fmt -p fret-node --check`: passed; proves the touched crate remains formatted.
  - `jq empty docs/workstreams/fret-node-declarative-fearless-refactor-v1/WORKSTREAM.json`: passed;
    proves the workstream metadata remains valid JSON.
  - `git diff --check`: passed; proves the patch has no whitespace errors.
  - `python3 tools/check_layering.py`: passed; proves the slice did not violate workspace layering
    policy.
  - `cargo check -p fret-node --all-features --tests`: passed; proves optional UI/integration test
    targets compile with update-anchor planning.
  - `cargo check -p fret-node --no-default-features`: passed; proves headless/runtime-facing
    package compilation remains unaffected by the UI-only update-anchor planning slice.
  - `cargo clippy -p fret-node --all-targets --all-features -- -D warnings`: passed; proves the
    update-anchor helper, frame diagnostics wiring, and tests are lint-clean.
  - `cargo nextest run -p fret-node`: passed; proves the full `fret-node` package suite remains
    green with 472 tests.

Fresh verification is required before marking a task, Codex goal, or lane complete.
