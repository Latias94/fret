# UI Gallery view-cache web perf stabilization (v1) — TODO

## Capture + compare loop

- [ ] Capture a baseline bundle with view-cache disabled (web).
- [ ] Capture an experimental bundle with `fret_ui_gallery_view_cache=1`.
- [ ] Capture a second experimental bundle with `fret_ui_gallery_view_cache_shell=1`.
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

