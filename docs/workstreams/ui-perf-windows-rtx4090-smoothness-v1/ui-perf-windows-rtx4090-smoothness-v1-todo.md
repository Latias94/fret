# UI perf: Windows RTX 4090 smoothness v1 — TODO

## P0 (gates / evidence)

- [x] Reclassify broad `ui-gallery-steady` Windows coverage as maintenance/evidence-only until the suite is split
  into narrower steady-contract groups.
  - Keep the daily smoke trio as the routine verification surface.
  - Use broad `ui-gallery-steady --repeat 7` runs only to collect tail evidence and confirm membership drift.
  - The attempted combined `ui-gallery-core-steady` baseline was rejected; do not revive it unless a later follow-on
    adds a stable narrower contract boundary.
- [x] Split `ui-gallery-overlay-steady` into narrower sub-contracts and keep the broad suite evidence-only.
  - `ui-gallery-context-menu-right-click-steady`, `ui-gallery-dialog-escape-focus-restore-steady`,
    `ui-gallery-dropdown-open-select-steady`, `ui-gallery-overlay-pointer-move-steady`, and
    `ui-gallery-overlay-torture-steady` are now split out and checked in as single-script baselines.
  - Keep `ui-gallery-overlay-steady` evidence-only until the broad suite drops the remaining modal/inspector/legacy members.
- [x] Promote the context-menu, dialog, dropdown, overlay pointer-move, and overlay-torture interaction follow-ons as dedicated single-script baselines.
- [x] Run `ui-resize-probes` with `--repeat 7` (resize jitter stability check).
- [x] Run `ui-code-editor-resize-probes` with `--repeat 7` (editor-class guardrail).
- [x] Establish a “typical perf” gate: `ui-gallery-complex-typical` baseline + `--perf-threshold-agg p90`.
- [ ] For any remaining outliers: capture one bundle with `--trace` and one with `FRET_LAYOUT_NODE_PROFILE=1`.

## Attribution loop (make spikes explainable)

- [x] Add a “standard diff recipe” section to this workstream (commands + expected outputs).
- [x] Identify 2–3 most common failing metrics on Windows (from `check.perf_thresholds.json`) and document “first place to look”.
- [x] Track one exemplar failure from each category:
  - [x] layout-root build spikes: `Finding (2026-02-14): repeat=7 can fail on Material3 tabs (request_build_roots dominates)`.
  - [x] layout-engine solve spikes: `Finding (2026-02-15): Batch-solve barrier roots to eliminate per-root solve spikes`.
  - [x] paint spikes: `Finding (2026-05-10): ui-gallery-complex-steady now yields a paint-dominant Windows exemplar when run with --prelude-each-run; use target/fret-diag/1778364986668/bundle.schema2.json for paint-tail attribution.`

## Instrumentation gaps (candidate fearless refactor items)

- [ ] Inventory “hot scratch structures” that can reallocate in spikes; add cheap grow counters (opt-in or always-on).
- [x] Add percentiles (p50/p95) for `diag stats` bundle summaries (typical perf review).
- [x] Export phase sub-events in `trace.chrome.json` derived from `debug.stats.*_time_us`.
- [x] Export per-run frame percentiles into `check.perf_thresholds.json` for quick scanning.
- [x] Add miss-only renderer spans for pipeline creation and intermediate target allocation/eviction.
- [ ] Make “phase timeline” → “top hotspots” linkage explicit in one place (docs + tool output).

## Windows-specific

- [x] Document PIX capture steps for `fret-ui-gallery.exe` (GPU-side sanity when CPU looks good).
- [x] Document ETW/WPR profile preset to correlate spikes with OS scheduling/IO.

## Known issues / stability

- [ ] Investigate occasional `thread 'main' has overflowed its stack` on `fret-ui-gallery.exe` exit after long perf suites.
