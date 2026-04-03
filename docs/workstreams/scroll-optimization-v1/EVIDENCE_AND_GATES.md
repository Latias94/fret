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

## Evidence anchors

- Seed / deferred policy / authoritative commit helpers:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- Mechanism regression coverage:
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- Contained relayout dirty/rerender bookkeeping:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
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
