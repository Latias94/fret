# Scroll Optimization Workstream (v1) — Evidence And Gates

Date: 2026-04-03  
Status: Active

## Current slice — Deferred probe seed vs authoritative extent

This slice locks the contract that:

- deferred probing can only happen when a retained seed extent exists,
- retained caches are seeds, not authoritative truth,
- pending probes clear only after an explicit probe or authoritative post-layout observation.

## Canonical gates

- Seed contract regression:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_deferred_invalidation_uses_intrinsic_cache_seed_before_measure`
- Authoritative observation clears deferred invalidation pending state:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_authoritative_observation_same_extent_clears_deferred_invalidation_pending_state`
- Budget-hit recovery (growth):
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_post_layout_budget_hit_growth_converges_via_pending_probe_next_frame`
- Budget-hit recovery (shrink):
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_post_layout_budget_hit_shrink_converges_via_pending_probe_next_frame`

## Evidence anchors

- Seed / deferred policy / authoritative commit helpers:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- Mechanism regression coverage:
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
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
