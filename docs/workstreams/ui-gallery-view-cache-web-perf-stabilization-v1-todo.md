# UI Gallery view-cache web perf stabilization (v1) — TODO

## Capture + compare loop

- [x] Capture a baseline bundle with view-cache disabled (web).
  - Evidence: `.fret/diag/exports/1771829809968-bundle`
- [x] Capture an experimental bundle with `fret_ui_gallery_view_cache=1`.
  - Evidence: `.fret/diag/exports/1771832191642-bundle` (view-cache active, but `view_cache_roots_total=0`)
- [ ] Capture view-cache harness bundles with URL-driven config (avoid UI toggle flake):
  - Scripts:
    - `tools/diag-scripts/ui-gallery-view-cache-harness-perf-steady-web-off.json`
    - `tools/diag-scripts/ui-gallery-view-cache-harness-perf-steady-web-on.json`
  - Expectation:
    - off: `view_cache_active=false`, `view_cache_roots_total>0` (has `cached_subtree`, but reuse should be gated off)
    - on: `view_cache_active=true`, `view_cache_roots_total>0`, `view_cache_roots_reused>0` after warm-up
  - Note: prefer `fret_ui_gallery_view_cache_continuous=1` on web so scripts can advance frames reliably.
- [x] Re-capture a view-cache bundle after:
  - enabling shell view-cache by default on `wasm32` when view-cache is enabled
  - removing per-frame model churn for undo/redo availability
  - Evidence: `.fret/diag/exports/1771842539046-bundle`
- [x] Confirm view-cache roots are actually mounted for the magic-patterns workload:
  - Observed: `view_cache_active=true`, `cache_roots=1`, `cache_roots_reused=1` (shell).
  - Note: the `magic_patterns_torture` content subtree intentionally remains uncached under view-cache to preserve animation.
- [ ] Compare:
  - `paint_time_us` p95 and max
  - `paint_cache_misses`
  - view-cache reuse counters (`cache_roots*`, `view_cache_active`)

## Interpretation gates

- [ ] If view-cache improves paint time (and does not introduce correctness regressions), adopt it for web perf evidence URLs used by renderer workstreams.
- [ ] If view-cache has low reuse or causes invalidation churn, diagnose:
  - unstable roots / changing model sources
  - over-broad invalidations
  - shell caching boundary (`shell.rs` view-cache roots)

## Evidence hygiene

- [ ] Record at least one “before vs after” bundle pair (paths only, no embedded tokens) in the milestones file.
- [ ] If a decision impacts `REN-VNEXT-webgpu-004`, add a short status note to `docs/workstreams/renderer-vnext-fearless-refactor-v1-todo.md`.
