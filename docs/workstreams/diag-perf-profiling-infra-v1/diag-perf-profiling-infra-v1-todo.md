# Diagnostics perf profiling infra v1 — TODO

## P0 (make the signals usable)

- [x] Add a compact "CPU delta vs wall delta" recipe to `docs/ui-diagnostics-and-scripted-tests.md`.
- [x] Print layout/paint sub-phase breakdown percentiles (`p50`/`p95`) in `diag stats` human output.
- [x] Add one example bundle + interpretation notes to `docs/workstreams/diag-perf-profiling-infra-v1/diag-perf-profiling-infra-v1.md`.
- [x] Ensure `diag stats --json` includes CPU cycle deltas in `top[]` rows (for tooling consumers).

## Contract & schema discipline

- [ ] Define a perf key registry (name/unit/kind/scope/aggregate).
  - [x] Seed the registry for trace-exported frame keys in `crates/fret-diag/src/perf_keys.rs`.
  - [x] Expose the registered stats/gate subset from `diag stats --json` via `registered_perf_keys`.
  - [ ] Expand the registry to all bundle/stats/gate fields before treating it as the single source of truth.
- [ ] Add contract tests that ensure:
  - [x] trace-exported keys are unique and include core timeline keys
  - [x] trace-exported key units are consistent (e.g. `*_time_us` is microseconds, `*_cycles` is cycles)
  - [x] registered stats/gate subset keys stay additive and unit-consistent
  - [ ] full registry keys are additive only (no accidental rename)
  - [ ] full registry units are consistent
- [x] Add a generated field inventory doc (or update `diag-perf-attribution-v1-field-inventory.md` from the registry).

## Tooling UX (shorten the attribution loop)

- [x] Add `diag stats --sort cpu_cycles` (or equivalent) to find frames where UI thread actually ran.
- [x] Add a `diag stats --diff` view that highlights both typical (p95) and tail (max) deltas.
- [x] Make `check.perf_thresholds.json` link to:
  - [x] worst bundle per failing metric
  - [x] optional trace artifact path when `--trace` is enabled

## Phase timeline coverage

- [x] Audit the current "frame timeline" coverage (layout/paint/dispatch/hit-test).
  - Evidence:
    `docs/workstreams/diag-perf-profiling-infra-v1/PHASE_TIMELINE_COVERAGE_AUDIT_2026-05-15.md`
  - Result: existing spans/timers cover the major runtime phases, but Chrome trace output remains
    bundle-derived synthetic phases and does not include live `tracing` / Tracy spans.
- [ ] Add missing always-on phase timers for known uninstrumented work (keep additive keys).
- [ ] Ensure chrome trace emits stable event names for new sub-phases.
- [ ] Adopt `crates/fret-perf` helpers for new/updated timers so stats + spans stay aligned.
  - [x] Migrate `layout_all` final tail phases through a shared `fret_perf::measure_span` path:
    regular frames, layout fast-path frames, and skipped-engine stable frames now share the
    prepaint/focus/semantics/deferred-cleanup timing and span wrapper surface.
    - Evidence: `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - [ ] Migrate more layout sub-phases beyond request/build + roots:
    - `crates/fret-ui/src/tree/layout/*.rs` (invalidate bindings, expand invalidations, contained roots, semantics refresh, etc.)
    - [x] Align `crates/fret-ui/src/layout/engine.rs` solve spans with `fret_perf::measure_span_with_finish`
      for batched independent-root solves and single-root solves while preserving `elapsed_us`,
      `measure_calls`, `measure_cache_hits`, `measure_us`, and existing solve/profile stats.
      - Evidence: `crates/fret-perf/src/lib.rs`, `crates/fret-ui/src/layout/engine.rs`
    - Remaining: per-widget measure hotspot timing is intentionally still a local debug profiling
      timer until a repro needs per-measure trace events; avoid per-node span explosion by default.
  - [x] Migrate remaining paint sub-phases and hot node paths:
    - [x] Migrate `paint_all` entry-layer sub-phases to `fret_perf::measure_span`:
      input context, scroll-handle invalidation, root collection, visual-bounds flush,
      text-input snapshot publish, and paint-observation collapse.
      - Evidence: `crates/fret-ui/src/tree/paint/entry.rs`
    - [x] Migrate `paint_node` hot-path timers to `fret_perf::measure_span` while preserving the
      existing debug stats buckets:
      visual-bounds record, cache key, cache hit check, cache bounds translate, widget paint,
      and paint-observation record.
      - Evidence: `crates/fret-ui/src/tree/paint/node.rs`
  - [ ] Extend runner/renderer phase spans where needed:
    - [x] Align `ecosystem/fret-bootstrap/src/ui_app_driver.rs` driver phases with
      `fret_perf::measure_span` while preserving frame-hitch log fields:
      view, overlay, layout, paint, and diagnostics script drive.
      - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
    - [x] Align native runner redraw phases with `fret_perf::measure_span` while preserving
      redraw-hitch log fields:
      prepare, render, record, present, and nested render scene.
      - Evidence: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
    - [x] Align web runner frame phases with `fret_perf::measure_span` using the same runner
      phase names:
      prepare, render, record, present, and nested render scene.
      - Evidence: `crates/fret-launch/src/runner/web/render_loop.rs`
    - `crates/fret-render-*` (prepare/record/submit/present boundaries)
