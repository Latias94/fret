# Scroll Optimization Workstream (v1) — Evidence And Gates

Date: 2026-04-03  
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

## Evidence anchors

- Seed / deferred policy / authoritative commit helpers:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- Mechanism regression coverage:
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
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
