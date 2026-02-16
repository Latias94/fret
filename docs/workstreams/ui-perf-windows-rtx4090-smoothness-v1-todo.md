# UI perf: Windows RTX 4090 smoothness v1 — TODO

## P0 (gates / evidence)

- [ ] Run `ui-gallery-steady` with `--repeat 7` (tail stability check) and archive worst bundles.
- [ ] Run `ui-resize-probes` with `--repeat 7` (resize jitter stability check).
- [ ] Run `ui-code-editor-resize-probes` with `--repeat 7` (editor-class guardrail).
- [x] Establish a “typical perf” gate: `ui-gallery-complex-typical` baseline + `--perf-threshold-agg p90`.
- [ ] For any remaining outliers: capture one bundle with `--trace` and one with `FRET_LAYOUT_NODE_PROFILE=1`.

## Attribution loop (make spikes explainable)

- [ ] Add a “standard diff recipe” section to this workstream (commands + expected outputs).
- [ ] Identify 2–3 most common failing metrics on Windows (from `check.perf_thresholds.json`) and document “first place to look”.
- [ ] Track one exemplar failure from each category:
  - [ ] layout-root build spikes
  - [ ] layout-engine solve spikes
  - [ ] paint spikes

## Instrumentation gaps (candidate fearless refactor items)

- [ ] Inventory “hot scratch structures” that can reallocate in spikes; add cheap grow counters (opt-in or always-on).
- [x] Add percentiles (p50/p95) for `diag stats` bundle summaries (typical perf review).
- [x] Export phase sub-events in `trace.chrome.json` derived from `debug.stats.*_time_us`.
- [x] Export per-run frame percentiles into `check.perf_thresholds.json` for quick scanning.
- [x] Add miss-only renderer spans for pipeline creation and intermediate target allocation/eviction.
- [ ] Make “phase timeline” → “top hotspots” linkage explicit in one place (docs + tool output).

## Windows-specific

- [ ] Document PIX capture steps for `fret-ui-gallery.exe` (GPU-side sanity when CPU looks good).
- [ ] Document ETW/WPR profile preset to correlate spikes with OS scheduling/IO.

## Known issues / stability

- [ ] Investigate occasional `thread 'main' has overflowed its stack` on `fret-ui-gallery.exe` exit after long perf suites.
