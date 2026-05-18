# fret-ui Layout Architecture Decision

Date: 2026-05-18
Task: FLA-030

## Decision

Do **not** redesign the layout model now.

The current clean-geometry classification axes are conceptually sound:

- `layout_effect`
- `child_bounds`
- `size_stability`
- explicit rejection attribution

The next architecture step should be a behavior-preserving organization refactor only if we are
about to keep expanding clean-geometry proofs. The preferred first code slice would be extracting
the private clean-geometry proof model and helper functions out of `tree/layout/node.rs` into a
dedicated private module, without changing behavior.

## Why not a model rewrite

The local FLA-020 resize-jitter sample does not show Taffy solve or text measurement as the dominant
cost:

- Worst frame: `total=2803us`, `layout=2304us`, `layout_roots=2181us`,
  `layout_engine_solve=202us`, `prepaint=202us`, `paint=297us`.
- p95 has the same shape because this was a one-repeat local orientation sample:
  `layout_engine_solve=202us`, `renderer_prepare_text_us=65us`, `paint_text_prepare_time_us=0`.
- ViewCache root reuse is stable: `cache_roots_reused=1/1`.
- Top solves are small `new_frame_key_changed` roots with no measure time:
  roughly `155us`, `43us`, and `3us`.
- Top layout hotspots point to retained-tree/barrier orchestration:
  `Semantics` inclusive around `2177us`, `Scroll` around `281us`, and `ViewCache` around `373us`.

This does not justify a broad node-kind redesign. It suggests the next perf owner, if any, is layout
orchestration around retained tree / barrier / `Scroll` / `ViewCache`, not the clean-geometry
classification model itself.

## What should happen next

Recommended next step:

1. If continuing architecture cleanup: run FLA-040 as a behavior-preserving extraction of
   clean-geometry helpers into a private module.
2. If chasing runtime performance: open a narrower follow-on for retained layout orchestration or
   root `Scroll` side-effect-boundary redesign, with fresh evidence and gates.
3. If working on text: continue through `text-intrinsic-sizing-and-wrap-v1`; do not widen
   clean-geometry to wrapped text here.

## What should not happen

- Do not keep widening `tree/layout/node.rs` with more proof cases by default.
- Do not treat wrapped `TextWrap::Word` as safe clean geometry without a dedicated line-break /
  computed-box stability proof.
- Do not skip `Scroll` layout by name; it is a side-effect boundary.
- Do not start a `measured_size: Option<Size>` migration unless zero-size ambiguity recurs outside
  already-explicit zero driver leaves.

## Evidence

- Source inventory:
  `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_INVENTORY_2026-05-18.md`
- Local perf bundle:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/1779077560550/bundle.schema2.json`
- Local perf stats:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/worst.stats.json`
- Prior closeout/handoff:
  `docs/workstreams/scroll-optimization-v1/HANDOFF.md`
