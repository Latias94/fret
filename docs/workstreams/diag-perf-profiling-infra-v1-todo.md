# Diagnostics perf profiling infra v1 — TODO

## P0 (make the signals usable)

- [x] Add a compact "CPU delta vs wall delta" recipe to `docs/ui-diagnostics-and-scripted-tests.md`.
- [ ] Add one example bundle + interpretation notes to `docs/workstreams/diag-perf-profiling-infra-v1.md`.
- [x] Ensure `diag stats --json` includes CPU cycle deltas in `top[]` rows (for tooling consumers).

## Contract & schema discipline

- [ ] Define a perf key registry (name/unit/kind/scope/aggregate).
- [ ] Add contract tests that ensure:
  - [ ] keys are additive only (no accidental rename)
  - [ ] units are consistent (e.g. `*_time_us` is always microseconds)
- [ ] Add a generated field inventory doc (or update `diag-perf-attribution-v1-field-inventory.md` from the registry).

## Tooling UX (shorten the attribution loop)

- [x] Add `diag stats --sort cpu_cycles` (or equivalent) to find frames where UI thread actually ran.
- [ ] Add a `diag stats --diff` view that highlights both typical (p95) and tail (max) deltas.
- [ ] Make `check.perf_thresholds.json` link to:
  - [ ] worst bundle per failing metric
  - [ ] optional trace artifact path when `--trace` is enabled

## Phase timeline coverage

- [ ] Audit the current "frame timeline" coverage (layout/paint/dispatch/hit-test).
- [ ] Add missing always-on phase timers for known uninstrumented work (keep additive keys).
- [ ] Ensure chrome trace emits stable event names for new sub-phases.
