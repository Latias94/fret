# Scroll Optimization Workstream (v1) — Evidence And Gates

Date: 2026-04-04  
Status: Active

## Current slice — Deferred probe seed vs authoritative extent

This slice locks the contract that:

- deferred probing can only happen when a retained seed extent exists,
- retained caches are seeds, not authoritative truth,
- pending probes clear only after an explicit probe or authoritative post-layout observation.

## Follow-on slice — Contained relayout dirty vs rerender semantics

This follow-on slice locks the contract that:

- contained relayout is a layout-only repair path, not an implicit “rerender next frame” request,
- `view_cache_needs_rerender` remains authoritative for declarative rerender pressure,
- scheduling-only dirty markers clear once layout invalidation and rerender pressure are both gone.

## Follow-on slice — Detached roots must not keep layout follow-up state alive

This follow-on slice locks the contract that:

- detached/unreachable cache roots must be pruned before contained-relayout candidate selection,
- detached/unreachable barrier roots must be pruned before pending barrier relayout execution,
- detached layout follow-up state must not block later stable frames from taking a layout-skip path.

## Follow-on slice — Barrier same-children fast path must still reach authoritative relayout

This follow-on slice locks the contract that:

- re-applying an unchanged barrier child list is still a no-op when the barrier subtree is clean,
- re-applying an unchanged barrier child list must schedule a contained barrier relayout when the
  barrier subtree still has pending layout work,
- descendant layout invalidations under a clean ancestor must not remain pinned just because the
  barrier child list was structurally unchanged.

## Follow-on slice — Contained cache-root dirty tracking must match authoritative layout state

This follow-on slice locks the contract that:

- a descendant layout invalidation truncated at a contained view-cache root must still make that
  root discoverable to the contained-relayout pass,
- layout-only descendant invalidations must not escalate a contained cache root into declarative
  rerender pressure,
- `dirty_cache_roots` must clear when authoritative main-pass layout already consumed the cache
  root's scheduling-only layout invalidation.

## Follow-on slice — Same-children parent repair must reconnect authoritative layout

This follow-on slice locks the contract that:

- `set_children(...same_children...)` remains a true no-op when parent pointers are already valid,
- `set_children(...same_children...)` must reconnect the parent into the authoritative layout
  invalidation walk when it repaired stale child parent pointers under pending descendant layout
  work,
- `set_children_in_mount(...same_children...)` must honor the same reconnect contract,
- `add_child(...)` must not bypass the same structural consistency contract: reparenting a child
  must sever the old parent's child edge, avoid duplicate child edges on the new parent, and route
  the resulting structural change through the authoritative layout invalidation path.

## Follow-on slice — Layer root replacement must prune detached interaction state

This follow-on slice locks the contract that:

- replacing a layer root must immediately clear `focus` / pointer captures that are no longer
  reachable from the current active input/focus roots,
- root replacement must preserve interaction state that remains reachable from another active layer
  root (for example, an overlay that stays mounted across base-root replacement),
- input-arbitration snapshots must reflect the pruned interaction state immediately instead of
  waiting for a later dispatch/command entry point to clean it up lazily.

## Follow-on slice — Pending shortcut continuation must revalidate authoritative key contexts

This follow-on slice locks the contract that:

- multi-stroke shortcut continuation must not rely on `focus` / `barrier_root` alone as a proxy
  for routing authority,
- if root replacement or another retained-tree repair changes the current key-context stack, the
  pending shortcut must be cleared before the next chord is matched,
- stale pending shortcut key contexts must not dispatch commands after the authoritative routing
  context changed.

## Follow-on slice — Cross-surface command gating snapshots must refresh key contexts

This follow-on slice locks the contract that:

- publishing command/action availability snapshots must also refresh the current
  `WindowKeyContextStackService` snapshot,
- cross-surface command gating must not keep stale `keyctx.*` values alive after rebuild/root
  replacement or other retained-tree reconfiguration,
- app/window-scope command gating must observe the same authoritative key-context stack as the
  current UI tree rather than the last input-event snapshot.

## Follow-on slice — Declarative rebuild commits must republish authoritative window snapshots

This follow-on slice locks the contract that:

- `render_root(...)` / `render_dismissible_root_with_hooks(...)` are authoritative declarative
  rebuild commit points for window-level snapshot consumers,
- once rebuild GC/root reuse has committed, later same-frame consumers must see refreshed
  `WindowInputContextService`, `WindowKeyContextStackService`, and
  `WindowCommandActionAvailabilityService` state,
- rebuild-time focus/key-context changes must revalidate pending shortcut state before stale
  overlay or gating consumers can observe it.

## Follow-on slice — Imperative tree mutations require explicit window snapshot commit

This follow-on slice locks the contract that:

- raw `UiTree` mutation APIs update retained tree state only; they do not silently republish
  window-level services,
- imperative mutation flows can make same-frame cross-surface consumers authoritative by calling
  `UiTree::publish_window_runtime_snapshots(...)`,
- the explicit commit surface must revalidate focus and pending shortcut/key-context state before
  writing `WindowInputContextService`, `WindowKeyContextStackService`, and
  `WindowCommandActionAvailabilityService`.

## Follow-on slice — Published input-context consumers must overlay authoritative command availability

This follow-on slice locks the contract that:

- `WindowInputContextService` remains a best-effort window snapshot transport for cross-surface
  consumers,
- when consumers need `edit.can_*` / `router.can_*` semantics, `WindowCommandAvailabilityService`
  stays authoritative and must overlay the published `InputContext`,
- stale published input snapshots must not suppress command gating or shortcut lookup once the
  authoritative command-availability service has changed.

## Follow-on slice — Scroll-handle revision-only bumps must preserve baseline vs window-update semantics

This follow-on slice locks the contract that:

- runtime-driven internal scroll-handle updates still commit offset/value baselines even when they
  do not bump the public revision,
- a later revision-only bump must remain a revision-only delta at the frame registry layer rather
  than being reclassified as a fresh offset change,
- final invalidation must still downgrade revision-only bumps to `HitTestOnly` by default,
- windowed-paint scroll surfaces stay reusable on revision-only bumps, while `VirtualList`
  surfaces can still escalate to cache-root window updates when the visible window escaped the
  rendered overscan window.

## Follow-on slice — Scroll-handle invalidation must ignore detached same-frame stale bindings

This follow-on slice locks the contract that:

- scroll-handle invalidation operates on the current live attached binding set, not a multiset of
  every same-frame registration that ever happened,
- detached/dead declarative nodes that are still present in retained same-frame bookkeeping must
  not receive scroll-handle invalidations,
- detached cache roots must not be dirtied by stale same-frame bindings,
- debug scroll-handle binding samples/counts must reflect the authoritative live attached nodes
  rather than stale or duplicate registrations.

## Follow-on slice — Scroll-handle registry writes must dedupe same-frame duplicate elements

This follow-on slice locks the contract that:

- repeated same-frame declarative registrations for the same `handle_key + element` pair must not
  accumulate duplicate registry entries,
- same-frame rebuilds may append new bindings for other elements, but repeated registrations of the
  same element must keep the registry set-like for that element,
- raw registry reads used by diagnostics/tests remain stable and do not grow with duplicate
  same-frame rebuilds.

## Follow-on slice — Event-time scroll-handle invalidation resolves authoritative live bindings

This follow-on slice locks the contract that:

- widget event handlers do not treat the raw scroll-handle registry as the authoritative invalidation
  target set,
- event-time scroll-handle invalidation requests are resolved by the dispatch/runtime layer after it
  regains access to `UiTree`,
- event-time invalidation still reaches live attached bindings across active layers,
- detached stale same-frame bindings remain ignored on the event path as well as the final
  invalidation path.

## Follow-on slice — Explicit scroll-target invalidation resolves authoritative live target nodes

This follow-on slice locks the contract that:

- mechanism widgets do not resolve explicit `scroll_target` elements by directly trusting
  `window_frame.instances.find_map(...)` during event handling,
- event-time `scroll_target` invalidation is deferred until dispatch regains access to `UiTree`,
- explicit scroll-target invalidation resolves live attached target nodes only,
- detached stale same-frame target entries do not win explicit scroll-target resolution.

## Follow-on slice — Command and event focus targets resolve authoritative live attached nodes

This follow-on slice locks the contract that:

- command dispatch must not treat `window_state.node_entry(element)` as the authoritative source
  node when pending command metadata only carries an element id,
- command hooks and event-side focus hooks may request focus by element, but the live attached node
  resolution must happen in `UiTree` / dispatch after runtime regains access to the authoritative
  retained tree,
- stale detached same-frame `node_entry` seeds must not win over a still-live attached node for the
  same element.

## Follow-on slice — Declarative rebuild and invalidation element paths resolve authoritative live nodes

This follow-on slice locks the contract that:

- declarative model/global/notify invalidation paths must not treat
  `window_state.node_entry(element)` as authoritative when `UiTree` is available,
- declarative mount/root reuse must prefer the live attached node for an element and only reuse a
  retained seed when no live attached node exists,
- view-cache GC / retained virtual-list reconcile roots must ignore detached stale `node_entry`
  seeds instead of keeping them alive as authoritative rebuild roots.

## Follow-on slice — Interaction targets resolve authoritative live attached nodes

This follow-on slice locks the contract that:

- hover/pressed/timer/selection runtime state may retain element identity across frames, but
  authoritative node resolution must happen against the live attached `UiTree` rather than by
  directly trusting a stale detached `node_entry(element)`,
- retained interaction target nodes are cache-like seeds that must be refreshed at final
  layout-frame commit, so a same-element rebuild/remount cannot keep clearing or dispatching to the
  old detached node,
- event helpers may carry `(element, node)` pairs when they already have a live dispatch target,
  but clearing or later dispatch must still consume the authoritative live node snapshot,
- selectable-text active-selection routing must keep targeting the live attached node even when
  retained selection state or `node_entry` was seeded with a stale detached node.

## Follow-on slice — Final-layout / dispatch / anchored queries resolve authoritative live attached nodes

This follow-on slice locks the contract that:

- render-time `focus-within` containment and focused-node-to-element sync are authoritative
  relation queries and must prefer the live declarative window frame before falling back to
  retained mappings,
- final-layout focus repair, touch-drag dispatch, wheel scroll-dismiss lookup, and anchored layout
  anchor-element lookup must not treat `elements::node_for_element(...)` as authoritative truth
  when `UiTree` or the declarative window frame is available,
- `elements::node_for_element(...)` remains a last-known post-frame / component-policy query
  surface; it is not the mechanism-layer source of truth for live attached nodes.

Audit note (2026-04-04):

- `rg -n "crate::elements::node_for_element\\(|elements::node_for_element\\(" crates/fret-ui/src --glob '!**/tests/**'`
  now returns no hits, so non-test mechanism paths in `crates/fret-ui` no longer resolve
  authoritative live nodes through the raw last-known query surface.

## Canonical gates

- Seed contract regression:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_deferred_invalidation_uses_intrinsic_cache_seed_before_measure`
- Authoritative observation clears deferred invalidation pending state:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_authoritative_observation_same_extent_clears_deferred_invalidation_pending_state`
- Budget-hit recovery (growth):
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_post_layout_budget_hit_growth_converges_via_pending_probe_next_frame`
- Budget-hit recovery (shrink):
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_post_layout_budget_hit_shrink_converges_via_pending_probe_next_frame`
- Contained relayout must not force next-frame rerender:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui view_cache_contained_relayout_does_not_force_next_frame_rerender`
- Layout-invalidated definite contained roots still allow reuse:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
- Explicit scroll-handle layout invalidations still force rerender:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
- Revision-only scroll-handle bumps after internal offset sync stay classified correctly:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui scroll_handle_revision_only_bumps_after_internal_offset_updates_classify_as_layout view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
- Detached same-frame stale scroll bindings are ignored:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui view_cache_scroll_handle_ignores_detached_same_frame_stale_bindings`
- Scroll-handle registry dedupes same-frame duplicate elements:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui scroll_handle_registry_dedupes_same_frame_duplicate_element_bindings`
- Event-time scroll-handle invalidation resolves authoritative live bindings across layers:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui event_scroll_handle_invalidation_targets_live_bindings_across_layers_only`
- Event-time explicit scroll-target invalidation resolves the live attached target:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui event_scroll_target_invalidation_prefers_live_attached_node_over_stale_same_frame_entry`
- Pending command source elements resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui dispatch_command_source_element_ignores_stale_detached_node_entry`
- Command hook focus requests resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui command_hooks_focus_request_ignores_stale_detached_node_entry`
- Key hook focus requests resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui key_hook_focus_request_ignores_stale_detached_node_entry`
- Pointer-region focus requests resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui declarative_pointer_region_focus_request_ignores_stale_detached_node_entry`
- Declarative model/global invalidation and rebuild seed resolution prefer live attached nodes:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui model_observation_invalidation_ignores_stale_detached_node_entry global_observation_invalidation_ignores_stale_detached_node_entry seeded_live_node_resolution_ignores_stale_detached_node_entry seeded_reusable_node_resolution_reuses_detached_seed_when_no_live_attached_node_exists`
- Hover/pressed/timer/selection interaction targets prefer live attached nodes over stale
  detached seeds:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui hovered_pressable_clear_uses_latest_node_for_same_element pressed_pressable_clear_uses_latest_node_for_same_element timer_dispatch_resolves_live_attached_element_target_over_stale_detached_seed final_layout_frame_syncs_hovered_pressable_node_to_live_attached_element selectable_text_set_text_selection_ignores_stale_detached_node_entry selectable_text_sets_active_text_selection`
- Render-time focus containment and focused-element sync prefer live window-frame nodes over stale
  detached seeds:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui element_context_reports_focus_within_for_focused_descendant element_context_focus_within_ignores_stale_detached_node_entries`
- Final-layout / dispatch / anchored live-node queries ignore stale detached seeds:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui focus_repair_prefers_live_attached_node_over_stale_detached_node_entry anchored_anchor_element_ignores_stale_detached_node_entry touch_pan_scroll_live_target_resolution_ignores_stale_detached_node_entry`
- Wheel scroll-dismiss lookup resolves the live attached element instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui dismissible_scroll_dismiss_ignores_stale_detached_node_entry`
- Detached dirty cache roots are pruned before contained relayout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui detached_dirty_view_cache_root_is_pruned_before_layout_followups`
- Detached pending barrier relayouts are pruned before execution:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui detached_pending_barrier_relayout_is_pruned_before_layout`
- Clean barrier same-children remounts stay no-op:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui set_children_barrier_same_children_clean_subtree_stays_noop`
- Dirty barrier same-children remounts still converge via authoritative relayout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui set_children_barrier_same_children_with_dirty_descendant_reaches_authoritative_relayout`
- Descendant layout invalidations still schedule contained relayout without rerender:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui descendant_layout_invalidation_marks_contained_view_cache_root_dirty`
- Same-children parent repair reconnects detached descendant layout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::children::set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
- Mount-time same-children parent repair reconnects detached descendant layout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::children::set_children_in_mount_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
- `add_child(...)` reparents without stale child edges and no-ops when already attached once:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui add_child_reparents_from_old_parent_without_leaving_stale_child_edges add_child_noops_when_child_is_already_attached_once_to_same_parent`
- Root replacement clears detached base-layer interaction state:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::layer_root_replacement::set_root_replacement_clears_detached_base_layer_interaction_state`
- Root replacement preserves still-active overlay interaction state:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::layer_root_replacement::set_root_replacement_preserves_overlay_interaction_state`
- Pending shortcut is cleared when root replacement changes the key-context stack:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
- Publishing action availability refreshes key-context snapshots for cross-surface gating:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
- Declarative rebuild refreshes window input snapshots before paint:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::render_root_rebuild_refreshes_window_input_context_snapshot_before_paint`
- Paint refreshes window input context after programmatic focus changes:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::window_input_context_snapshot::paint_all_publishes_programmatic_input_context_snapshot`
- Declarative rebuild refreshes window key-context snapshots before the next explicit publish:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::render_root_rebuild_refreshes_window_key_context_snapshot_before_next_publish`
- Declarative rebuild refreshes widget command availability before the next explicit publish:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::render_root_rebuild_refreshes_command_action_availability_before_next_publish`
- Imperative tree mutation refreshes window input context only after explicit snapshot commit:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_input_context`
- Imperative tree mutation refreshes key-context snapshots only after explicit snapshot commit:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_key_contexts`
- Imperative tree mutation refreshes widget command availability only after explicit snapshot commit:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_command_availability`
- Best-effort input context overlays authoritative command availability:
  - `cargo nextest run -p fret-runtime best_effort_input_context_overlays_authoritative_command_availability`
- Best-effort input-context fallback inherits authoritative command availability:
  - `cargo nextest run -p fret-runtime best_effort_input_context_fallback_inherits_command_availability`
- Window command-gating fallback overlays authoritative command availability over stale input snapshots:
  - `cargo nextest run -p fret-runtime snapshot_for_window_overlays_authoritative_command_availability_over_stale_input_context`
- shadcn shortcut display prefers authoritative command availability over stale published input snapshots:
  - `cargo nextest run -p fret-ui-shadcn shortcut_display_input_context_prefers_authoritative_command_availability`
- Source-policy gate forbids raw window input snapshots from bypassing command-availability helpers:
  - `python3 tools/check_window_input_context_command_availability_usage.py`

## Evidence anchors

- Seed / deferred policy / authoritative commit helpers:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- Scroll-handle baseline commit / revision classification:
  - `crates/fret-ui/src/declarative/frame.rs`
- Live attached scroll-handle binding resolution:
  - `crates/fret-ui/src/tree/layout/state.rs`
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
- Immediate event/paint scroll-handle binding consumers:
  - `crates/fret-ui/src/declarative/host_widget/event/mod.rs`
  - `crates/fret-ui/src/declarative/host_widget/paint.rs`
- Mechanism regression coverage:
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
- Final scroll-handle invalidation / window-update escalation coverage:
  - `crates/fret-ui/src/tree/layout/state.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- Child-list mutation helper coverage:
  - `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
  - `crates/fret-ui/src/tree/tests/children.rs`
- Best-effort window snapshot / command-availability overlay helpers:
  - `crates/fret-runtime/src/window_input_context.rs`
  - `crates/fret-runtime/src/window_command_gating/helpers.rs`
- Cross-surface consumer regression coverage:
  - `crates/fret-runtime/src/window_command_gating/tests.rs`
  - `ecosystem/fret-ui-shadcn/src/shortcut_display.rs`
- Source-policy guardrail:
  - `tools/check_window_input_context_command_availability_usage.py`
  - `.github/workflows/consistency-checks.yml`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- Contained relayout dirty/rerender bookkeeping:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
- Detached follow-up pruning + regression coverage:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
  - `crates/fret-ui/src/tree/tests/barrier_subtree_layout_dirty_aggregation.rs`
- Barrier same-children follow-up scheduling:
  - `crates/fret-ui/src/tree/ui_tree_mutation/barrier.rs`
  - `crates/fret-ui/src/tree/tests/barrier_subtree_layout_dirty_aggregation.rs`
- Contained cache-root dirty-marker lifecycle:
  - `crates/fret-ui/src/tree/layout/node.rs`
  - `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- Same-children parent repair reconnect path:
  - `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/mount.rs`
  - `crates/fret-ui/src/tree/tests/children.rs`
- Layer-root replacement interaction pruning:
  - `crates/fret-ui/src/tree/layers/impls.rs`
  - `crates/fret-ui/src/tree/tests/layer_root_replacement.rs`
- Pending shortcut authoritative-context revalidation:
  - `crates/fret-ui/src/tree/dispatch/window.rs`
  - `crates/fret-ui/src/tree/shortcuts.rs`
- Cross-surface command gating key-context snapshot refresh:
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/tree/tests/window_command_action_availability_snapshot.rs`
- Declarative rebuild window-snapshot republish:
  - `crates/fret-ui/src/declarative/mount.rs`
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/declarative/tests/core.rs`
- Imperative window-snapshot commit surface:
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/tree/dispatch/window.rs`
  - `crates/fret-ui/src/tree/paint/entry.rs`
  - `crates/fret-ui/src/tree/tests/window_input_context_snapshot.rs`
  - `crates/fret-ui/src/declarative/tests/core.rs`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Lane positioning:
  - `docs/workstreams/scroll-optimization-v1/DESIGN.md`
  - `docs/workstreams/scroll-optimization-v1/TODO.md`

## Verification notes

- 2026-04-03: compile gate confirmed with
  `CARGO_TARGET_DIR=target-codex-verify3 cargo check -p fret-ui --tests`.
- 2026-04-03: dedicated test binary linked successfully with
  `CARGO_TARGET_DIR=target-codex-verify3 cargo test -p fret-ui --lib --no-run`.
- 2026-04-03: targeted execution gates confirmed via
  `target-codex-verify3/debug/deps/fret_ui-c0a3056b7a68a9e7 --exact ...`:
  - `declarative::tests::layout::scroll::scroll_deferred_invalidation_uses_intrinsic_cache_seed_before_measure`
  - `declarative::tests::layout::scroll::scroll_authoritative_observation_same_extent_clears_deferred_invalidation_pending_state`
  - `declarative::tests::layout::scroll::scroll_post_layout_budget_hit_growth_converges_via_pending_probe_next_frame`
  - `declarative::tests::layout::scroll::scroll_post_layout_budget_hit_shrink_converges_via_pending_probe_next_frame`
- 2026-04-03: follow-on contained-relayout gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify4`:
  - `tree::tests::view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
  - `tree::tests::view_cache::view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
  - `tree::tests::view_cache::view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
- 2026-04-03: revision-only scroll-handle classification gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-ui`:
  - `declarative::frame::tests::scroll_handle_revision_only_bumps_after_internal_offset_updates_classify_as_layout`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only`
  - `tree::tests::view_cache::view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
  - `tree::tests::view_cache::view_cache_scroll_handle_window_update_marks_cache_root_needs_rerender`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_marks_cache_root_needs_rerender`
- 2026-04-04: live-binding filtering for same-frame stale scroll registrations confirmed via
  `cargo nextest` with `CARGO_TARGET_DIR=target-codex-ui`:
  - `tree::tests::view_cache::view_cache_scroll_handle_ignores_detached_same_frame_stale_bindings`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only`
  - `tree::tests::view_cache::view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
  - `tree::tests::view_cache::view_cache_scroll_handle_window_update_marks_cache_root_needs_rerender`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_marks_cache_root_needs_rerender`
  - `declarative::frame::tests::scroll_handle_revision_only_bumps_after_internal_offset_updates_classify_as_layout`
- 2026-04-04: same-frame duplicate scroll binding registrations dedupe correctly via
  `cargo nextest` with `CARGO_TARGET_DIR=target-codex-ui`:
  - `declarative::frame::tests::scroll_handle_registry_dedupes_same_frame_duplicate_element_bindings`
  - `tree::tests::view_cache::view_cache_scroll_handle_ignores_detached_same_frame_stale_bindings`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only`
  - `tree::tests::view_cache::view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
- 2026-04-03: detached-root follow-up pruning gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify5`:
  - `tree::tests::view_cache::detached_dirty_view_cache_root_is_pruned_before_layout_followups`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::detached_pending_barrier_relayout_is_pruned_before_layout`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
- 2026-04-03: barrier same-children follow-up gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify6`:
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::set_children_barrier_same_children_clean_subtree_stays_noop`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::set_children_barrier_same_children_with_dirty_descendant_schedules_barrier_relayout`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::set_children_barrier_same_children_with_dirty_descendant_reaches_authoritative_relayout`
  - `tree::tests::view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
  - `tree::tests::view_cache::view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
  - `tree::tests::view_cache::view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
- 2026-04-03: contained cache-root dirty-marker lifecycle gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify7`:
  - `tree::tests::view_cache::view_cache_invalidation_stops_at_boundary_for_paint`
  - `tree::tests::view_cache::descendant_layout_invalidation_marks_contained_view_cache_root_dirty`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
  - `tree::tests::view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender`
  - `tree::tests::view_cache::view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
  - `tree::tests::view_cache::view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
  - `tree::tests::view_cache::detached_dirty_view_cache_root_is_pruned_before_layout_followups`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::*`
- 2026-04-03: same-children parent-repair reconnect gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify8`:
  - `tree::tests::children::set_children_noops_when_unchanged`
  - `tree::tests::children::set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
  - `tree::tests::children::set_children_in_mount_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::*`
  - `tree::tests::view_cache::*` targeted contained-relayout gates
- 2026-04-03: remaining child-list mutation helper audit closed with `add_child(...)` now routed
  through the same authoritative child-list contract via `cargo nextest`:
  - `tree::tests::children::add_child_reparents_from_old_parent_without_leaving_stale_child_edges`
  - `tree::tests::children::add_child_noops_when_child_is_already_attached_once_to_same_parent`
  - `tree::tests::children::set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
  - `tree::tests::children::set_children_in_mount_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
- 2026-04-03: layer-root replacement interaction-pruning gates confirmed via `cargo nextest`:
  - `tree::tests::layer_root_replacement::set_root_replacement_clears_detached_base_layer_interaction_state`
  - `tree::tests::layer_root_replacement::set_root_replacement_preserves_overlay_interaction_state`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_event_publishes_post_dispatch_input_arbitration_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_command_publishes_post_dispatch_input_arbitration_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::modal_barrier_scopes_pointer_capture_to_active_roots`
  - `tree::tests::semantics_focus_shortcuts::remove_layer_uninstalls_overlay_and_removes_subtree`
- 2026-04-03: pending-shortcut authoritative-context revalidation gates confirmed via `cargo nextest`:
  - `tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
  - `tree::shortcuts::tests::pending_sequence_matches_reserved_second_chord_before_text_input_consumes`
  - `tree::tests::command_enabled_service::shortcut_dispatch_respects_window_command_enabled_service`
  - `tree::tests::command_enabled_service::shortcut_dispatch_respects_window_command_action_availability_snapshot`
  - `tree::tests::command_enabled_service::focus_menu_bar_shortcut_dispatches_when_menu_bar_focus_service_is_present`
- 2026-04-03: cross-surface command-gating key-context refresh gates confirmed via `cargo nextest`:
  - `tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
  - `tree::tests::window_command_action_availability_snapshot::action_availability_snapshot_marks_unhandled_commands_unavailable`
  - `tree::tests::window_command_action_availability_snapshot::action_availability_snapshot_publishes_focus_traversal_gating`
  - `tree::tests::window_command_action_availability_snapshot::action_availability_snapshot_publishes_focus_menu_bar_gating`
  - `tree::tests::window_command_action_availability_snapshot::dispatch_event_publishes_action_availability_snapshot`
- 2026-04-03: declarative rebuild window-snapshot republish gates confirmed via `cargo nextest`:
  - `declarative::tests::core::render_root_rebuild_refreshes_window_input_context_snapshot_before_paint`
  - `declarative::tests::core::render_root_rebuild_refreshes_window_key_context_snapshot_before_next_publish`
  - `declarative::tests::core::render_root_rebuild_refreshes_command_action_availability_before_next_publish`
  - `tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
  - `tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
  - `tree::tests::window_input_context_snapshot::dispatch_event_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_context_snapshot::dispatch_command_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_event_publishes_post_dispatch_input_arbitration_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_command_publishes_post_dispatch_input_arbitration_snapshot`
- 2026-04-03: imperative window-snapshot commit gates confirmed via `cargo nextest`:
  - `declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_input_context`
  - `declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_key_contexts`
  - `declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_command_availability`
  - `declarative::tests::core::render_root_rebuild_refreshes_window_input_context_snapshot_before_paint`
  - `declarative::tests::core::render_root_rebuild_refreshes_window_key_context_snapshot_before_next_publish`
  - `declarative::tests::core::render_root_rebuild_refreshes_command_action_availability_before_next_publish`
  - `tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
  - `tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
  - `tree::tests::window_command_action_availability_snapshot::dispatch_event_publishes_action_availability_snapshot`
  - `tree::tests::window_input_context_snapshot::dispatch_event_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_context_snapshot::dispatch_command_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_context_snapshot::paint_all_publishes_programmatic_input_context_snapshot`
- 2026-04-03: best-effort input-context authoritative-overlay runtime gates confirmed via `cargo nextest`:
  - `best_effort_input_context_overlays_authoritative_command_availability`
  - `best_effort_input_context_fallback_inherits_command_availability`
  - `snapshot_for_window_overlays_authoritative_command_availability_over_stale_input_context`
- 2026-04-03: source-policy guardrail added so raw `WindowInputContextService` reads cannot feed
  command/shortcut consumers or own `edit.can_*` / `router.can_*` truth outside the runtime
  publisher/helper allowlist:
  - `python3 tools/check_window_input_context_command_availability_usage.py`
  - `.github/workflows/consistency-checks.yml`
- 2026-04-03: remaining raw `WindowInputContextService` consumers audited after the
  command-availability overlay refactor:
  - runtime owner/publisher sites remain in `crates/fret-ui/src/tree/commands.rs` and
    `crates/fret-runtime/src/window_input_context.rs`,
  - diagnostics readers in `ecosystem/fret-bootstrap/src/ui_diagnostics/{service.rs,script_steps_wait.rs,script_steps_assert.rs,script_steps_visibility.rs,script_steps_drag.rs}`
    use the snapshot only for window liveness, `focus_is_text_input`, and platform capability
    predicates, not command-availability truth,
  - text/IME readers in `ecosystem/fret-code-editor/src/editor/mod.rs` and
    `apps/fret-ui-gallery/src/ui/previews/pages/editors/web_ime.rs` use the snapshot only for
    `text_boundary_mode` / `focus_is_text_input`,
  - no remaining first-party command/shortcut consumers bypass the runtime helper overlay; the
    source-policy gate now enforces that boundary.
