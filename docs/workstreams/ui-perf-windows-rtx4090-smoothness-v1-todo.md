# UI perf: Windows RTX 4090 smoothness v1 — TODO

## P0 (gates / evidence)

- [ ] Run `ui-gallery-steady` with `--repeat 7` (tail stability check) and archive worst bundles.
- [ ] Run `ui-resize-probes` with `--repeat 7` (resize jitter stability check).
- [ ] Run `ui-code-editor-resize-probes` with `--repeat 7` (editor-class guardrail).
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
- [ ] Add optional percentiles (p50/p95/p99) for `diag stats` bundle summaries (typical perf review).
- [ ] Make trace artifact naming stable and searchable (“phase timeline” + “top hotspots” linkage).

## Windows-specific

- [ ] Document PIX capture steps for `fret-ui-gallery.exe` (GPU-side sanity when CPU looks good).
- [ ] Document ETW/WPR profile preset to correlate spikes with OS scheduling/IO.

