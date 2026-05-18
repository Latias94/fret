# VCRJ-020 Source Attribution

Date: 2026-05-18
Task: VCRJ-020
Status: Complete

## Summary

The starting hotspot is a real `ViewCache` layout-node hotspot, but the current evidence does not
support an allowlist-style runtime change.

The important split is:

- The dedicated `fret.ui.layout.view_cache` phase is small in the starting bundle
  (`layout_view_cache_time_us ~= 29-30us`).
- The `ViewCache layout_us=380 inclusive_us=723` hotspot is recorded during the main
  `layout_roots` pass, where the root `ViewCache` host widget is laid out as part of ordinary
  bounds propagation.
- The frame records `view_cache_roots_reused=1`, `view_cache_contained_relayouts=0`, and
  `view_cache_roots_layout_invalidated=0`, so this is not currently a contained-relayout owner.
- The clean-geometry rejection in the same sample starts at `Text/text_reflow`, not at
  `ViewCache/side_effect_boundary`.

## Starting Bundle Read

Source artifact:

- `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json`
- `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json`

Worst-frame evidence:

```text
frame_id=450
layout_time_us=897
layout_roots_time_us=764
layout_view_cache_time_us=30
layout_repair_view_cache_bounds_time_us=1
layout_contained_view_cache_roots_time_us=0
layout_collapse_layout_observations_time_us=29
view_cache_roots_reused=1
view_cache_contained_relayouts=0
view_cache_roots_layout_invalidated=0
layout_clean_geometry_solve_skip_first_element_kind=Text
layout_clean_geometry_solve_skip_first_rejection=text_reflow
```

Hotspots:

```text
ViewCache layout_us=380 inclusive_us=723
Scroll layout_us=205 inclusive_us=331
Flex layout_us=83 inclusive_us=122
```

Interpretation:

- `layout_view_cache_time_us` is not the source of the tail. It is only the post-root view-cache
  phase (`repair_bounds`, `layout_contained_roots`, `collapse_observations`).
- The `ViewCache` node hotspot is rooted at
  `apps/fret-ui-gallery/src/driver/shell.rs:164:12[slot=0]`, which means the UI Gallery shell
  view-cache root is still entering host-widget layout during window-size changes.
- `layout_roots_time_us` dominates the frame, and the clean-geometry skip is blocked by text
  reflow in the subtree. This makes a `ViewCache` allowlist change a weak first move.

## Source Ownership Map

### Declarative authoring and reuse

`ElementContext::view_cache(...)` owns the authoring-level cache-hit decision:

- `crates/fret-ui/src/elements/cx.rs:1451`
- It checks `should_reuse_view_cache_node(node)`, computes a key from theme, scale, explicit
  `cache_key`, environment deps, and layout-query deps.
- On reuse it marks the root as a reuse root, touches state keys and observed deps, and emits a
  `ViewCache` element with no children.
- On miss it runs the child closure, records deps, and writes the next cache key.

This is not just visual wrapper logic; cache-hit frames still need liveness and dependency
bookkeeping.

### Build-time liveness and identity

`ViewCacheBuildBoundaryStore` owns global-element keyed cache-boundary state:

- `crates/fret-ui/src/elements/runtime.rs:232`
- It stores rendered/next cache keys, state keys, authoring identities, subtree elements, reuse
  roots, key mismatches, and transition frames.
- `touch_view_cache_state_keys_if_recorded(...)`,
  `touch_view_cache_authoring_identities_if_recorded(...)`, and
  `touch_view_cache_action_hook_state_for_subtree_elements(...)` preserve state and action-hook
  liveness across skipped child closures.

This is a real side-effect surface. Any layout fast path that treats `ViewCache` as an ordinary
pure wrapper must prove it does not suppress these refresh paths.

### Mount-time boundary state

`mount_element(...)` binds `ViewCache` element state to retained nodes:

- `crates/fret-ui/src/declarative/mount.rs:1470`
- `reuse_view_cache` is derived from `window_state.should_reuse_view_cache_root(id)`.
- A reused cache root with deferred scroll work can still get `Invalidation::Layout`.
- `set_node_view_cache_flags(...)` records whether the boundary contains layout when bounds are
  known and whether its size is definite.
- Membership refresh paths keep ancestor cache-root membership and recorded state dependencies live
  across reuse frames.

Mount therefore owns cache-boundary identity, not only child list replacement.

### Runtime boundary metadata

`ViewBoundaryState` owns node-keyed boundary semantics:

- `crates/fret-ui/src/tree/view_boundary.rs:48`
- `BoundaryLayoutDependencies::allows_contained_relayout()` is true only for
  `ContainedWhenBoundsKnown` plus definite layout.
- `boundary_allows_contained_relayout(...)` feeds `should_reuse_view_cache_node(...)`.

This is the guard that distinguishes "cache-hit can skip declarative rerender" from "layout dirty
can be safely handled as contained layout".

### Layout entrypoints

`layout_all_with_pass_kind(...)` separates four view-cache-related surfaces:

- `expand_view_cache_layout_invalidations_if_needed(...)` before root layout.
- Main `layout_roots` pass, where the starting hotspot is currently recorded.
- `repair_view_cache_root_bounds_from_engine_if_needed(...)`,
  `layout_contained_view_cache_roots_if_needed(...)`, and
  `collapse_layout_observations_to_view_cache_roots_if_needed(...)` inside the dedicated
  `fret.ui.layout.view_cache` phase.
- Follow-up scroll relayout after contained view-cache relayout.

Starting evidence shows the dedicated phase is not dominant. The main root pass remains dominant.

### Host-widget geometry

`ElementHostWidget::layout_impl(...)` gives `ViewCache` wrapper-like geometry:

- `crates/fret-ui/src/declarative/host_widget/layout.rs:372`
- It first tries layout-engine/manual absolute child placement.
- Otherwise it falls back to `layout_positioned_container_impl(...)`, the same shape used by
  `Pressable` and `Semantics`.

This makes a future narrow fast path plausible only for a cache-hit, clean, bounds-only resize
case. It does not make `ViewCache` generally pure.

### Clean-geometry contract

`clean_geometry_node_contract(...)` intentionally classifies `ViewCache` as a side-effect boundary:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs:764`
- The execution allowlist does not include `ViewCache`:
  `crates/fret-ui/src/tree/layout/clean_geometry.rs:2179`.

This should remain unchanged until a focused proof demonstrates a narrower safe case.

## Verdict

No runtime change in VCRJ-020.

The current owner verdict is:

- `ViewCache` is the top layout-node hotspot in the starting bundle.
- The cost is attributed to main root layout, not contained view-cache relayout.
- The most promising next question is not "can `ViewCache` be globally pure?", but "why does the
  UI Gallery shell cache root still enter host-widget layout during clean resize, and is the
  blocker actually text reflow below it?"

## Recommended Next Task

Run VCRJ-030 with a fresh bundle and keep the following fields in the summary:

- `layout_roots_time_us`
- `layout_view_cache_time_us`
- `layout_repair_view_cache_bounds_time_us`
- `layout_contained_view_cache_roots_time_us`
- `layout_collapse_layout_observations_time_us`
- `view_cache_roots_reused`
- `view_cache_contained_relayouts`
- `view_cache_roots_layout_invalidated`
- `layout_clean_geometry_solve_skip_first_element_kind`
- `layout_clean_geometry_solve_skip_first_rejection`
- `layout_hotspots`
- `top_layout_engine_solves`

If the fresh bundle repeats the same signature, VCRJ-040 should start with a focused RED proof for
a cache-hit, clean root-bounds-only resize where `ViewCache` remains reused and no contained
relayout or rerender is requested. If the first rejection remains `Text/text_reflow`, split a text
reflow clean-geometry lane instead of changing `ViewCache`.
