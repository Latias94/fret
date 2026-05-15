# Diagnostics perf profiling infra v1 — TODO

## P0 (make the signals usable)

- [x] Add a compact "CPU delta vs wall delta" recipe to `docs/ui-diagnostics-and-scripted-tests.md`.
- [x] Print layout/paint sub-phase breakdown percentiles (`p50`/`p95`) in `diag stats` human output.
- [ ] Add one example bundle + interpretation notes to `docs/workstreams/diag-perf-profiling-infra-v1/diag-perf-profiling-infra-v1.md`.
- [x] Ensure `diag stats --json` includes CPU cycle deltas in `top[]` rows (for tooling consumers).

## Contract & schema discipline

- [ ] Define a perf key registry (name/unit/kind/scope/aggregate).
  - [x] Seed the registry for trace-exported frame keys in `crates/fret-diag/src/perf_keys.rs`.
  - [ ] Expand the registry to all bundle/stats/gate fields before treating it as the single source of truth.
- [ ] Add contract tests that ensure:
  - [x] trace-exported keys are unique and include core timeline keys
  - [x] trace-exported key units are consistent (e.g. `*_time_us` is microseconds, `*_cycles` is cycles)
  - [ ] full registry keys are additive only (no accidental rename)
  - [ ] full registry units are consistent
- [ ] Add a generated field inventory doc (or update `diag-perf-attribution-v1-field-inventory.md` from the registry).

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
  - [ ] Migrate more layout sub-phases beyond request/build + roots:
    - `crates/fret-ui/src/tree/layout/*.rs` (invalidate bindings, expand invalidations, contained roots, semantics refresh, etc.)
    - `crates/fret-ui/src/layout/engine.rs` (solve/measure sub-spans, if we want tighter attribution)
  - [ ] Migrate remaining paint sub-phases and hot node paths:
    - `crates/fret-ui/src/tree/paint/entry.rs` (input ctx, cache replay, etc.)
    - `crates/fret-ui/src/tree/paint/node.rs` (cache key, hit check, replay/translate, widget paint)
  - [ ] Extend runner/renderer phase spans where needed:
    - `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (frame phase spans)
    - `crates/fret-render-*` (prepare/record/submit/present boundaries)
