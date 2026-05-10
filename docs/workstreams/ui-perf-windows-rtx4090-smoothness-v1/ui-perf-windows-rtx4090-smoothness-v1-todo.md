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
- [x] Promote a dedicated code-editor autoscroll typical gate:
  `ui-gallery-code-editor-torture-autoscroll-typical` + `--perf-threshold-agg p90`, with UI-only `frame_p95_*`
  thresholds. Keep the steady v2 gate responsible for renderer all-surface tail checks.
- [x] For any remaining outliers: capture one bundle with `--trace` and one with `FRET_LAYOUT_NODE_PROFILE=1`.
  - Trace bundle: `target/fret-diag/perf-code-editor-hosted-resources-trace-v1/1778449929019/bundle.schema2.json`
    (the run directory also includes `trace.chrome.json`; p50/p95/max total `1603/1722/1722us`).
  - Layout-node profile bundle: `target/fret-diag/perf-code-editor-hosted-resources-layout-node-profile-v1/1778450026275/bundle.schema2.json`
    (p50/p95/max total `1723/1730/1730us`; hotspots centered on `Scroll` nodes in `scroll_area.rs` / `content.rs`).

## Attribution loop (make spikes explainable)

- [x] Add a “standard diff recipe” section to this workstream (commands + expected outputs).
- [x] Identify 2–3 most common failing metrics on Windows (from `check.perf_thresholds.json`) and document “first place to look”.
- [x] Track one exemplar failure from each category:
  - [x] layout-root build spikes: `Finding (2026-02-14): repeat=7 can fail on Material3 tabs (request_build_roots dominates)`.
  - [x] layout-engine solve spikes: `Finding (2026-02-15): Batch-solve barrier roots to eliminate per-root solve spikes`.
  - [x] paint spikes: `Finding (2026-05-10): ui-gallery-complex-steady now yields a paint-dominant Windows exemplar when run with --prelude-each-run; use target/fret-diag/1778364986668/bundle.schema2.json for paint-tail attribution.`

## Instrumentation gaps (candidate fearless refactor items)

- [ ] Inventory “hot scratch structures” that can reallocate in spikes; add cheap grow counters (opt-in or always-on).
  - [x] `scratch_element_children_vec_pool` now reports `grow_events` through `UiDebugFrameStats`, `ElementDiagnosticsSnapshotV1`, `diag stats`, and `memory_summary`; the new `fret-diag` stats parser test passes.
- [x] Add renderer encode family attribution behind `FRET_DIAG_RENDERER_ENCODE_FAMILY_PROFILE=1` so
  `renderer_encode_scene_us` can be split into stack/clip/mask/effect/quad/image/text/path/viewport/flush buckets in
  `diag stats` and triage JSON.
- [x] Capture one fresh bundle with the renderer encode family profile enabled and use it to decide the next
  renderer-side follow-on. The current probe shows `text` dominates the remaining encode tail.
- [x] Follow up on the text-heavy renderer encode path and decide whether the next slice is text vertex construction,
  text batching, or scene text indexing. The first slice added text-phase attribution and moved atlas page/UV lookup out
  of the encode loop with a `TextShape` render-glyph cache.
- [x] Add a translation+uniform-scale fast path for text glyph transforms in the renderer encode loop. The
  `ui-gallery-code-editor-torture-autoscroll-steady` probe now reports
  `encode_scene_text_transform_fast_path_glyphs=20420`, `encode_scene_text_transform_generic_glyphs=0`, and
  `encode_scene_text_vertex_grow_events=0`, with repeat=3 p50/p95/max total `2091/2165/2165us` and paint
  `1893/1937/1937us` in `target/fret-diag/perf-code-editor-render-cache-v4/1778420924991/bundle.schema2.json`.
- [x] Attribute/reduce the remaining text vertex emission, group-bounds, and group-flush work in the renderer encode
  path before promoting a deeper row/fragment replay refactor. Follow-up evidence showed row-scene replay already hits
  `288/289` visible rows on the steady autoscroll probe. The apparent `~900us` glyph bucket under
  `FRET_DIAG_RENDERER_ENCODE_FAMILY_PROFILE=1` mostly came from per-glyph diagnostic timing, so
  `FRET_DIAG_RENDERER_TEXT_GLYPH_EMIT_PROFILE=1` now gates that detailed timer. The repeat=7 low-overhead family-profile
  gate passed at p50/p95/max total `2019/2486/2486us`, with worst-bundle renderer encode p95/max `361/361us` and
  `renderer.encode.text(us/transform/emit/flush)=0/0/17-22us`.
- [ ] Re-evaluate row/fragment replay only from a fresh low-overhead profile. If steady editor rows still rebuild too
  much text or geometry after hosted-resource touch precompute and default family profiling, prototype the smallest
  row-scoped replay cache and keep the code-editor v2 steady baseline as the guardrail.
  - 2026-05-11 content-resolve probe note: key comparison and key construction are already tiny relative to the
    row-scene fast replay path. If this lane revisits row-scoped replay work, target replay/touch mechanics or text
    draw before spending time on more RowGeomKey / RowSceneKey splitting.
  - 2026-05-11 hosted-resource precompute note: `CanvasHostedResources` moved the hosted-resource touch scan off the
    replay hit path. The next probe should keep measuring replay/touch mechanics or new-row text draw, not more key
    splitting.
- [x] Smooth syntax-cache miss spikes on the code-editor paint path (prefetch or background fill) using
  `ui-gallery-code-editor-torture-autoscroll-steady` as the guardrail. Current telemetry shows a single syntax miss can
  add ~4.2ms to a frame (`tick=341` in
  `target/fret-diag/perf-code-editor-paint-telemetry/1778386820783/bundle.schema2.json`), and the follow-up breakdown
  shows the miss is dominated by synchronous Tree-sitter highlight (`us_syntax_highlight=4069us` of
  `us_syntax_spans=4316us` in
  `target/fret-diag/perf-code-editor-paint-telemetry-syntax-breakdown3/1778389032255/bundle.schema2.json`). Syntax
  prefetch now removes highlight from the paint frame (`us_syntax_highlight=0` in
  `target/fret-diag/perf-code-editor-syntax-prefetch/1778392044589/bundle.schema2.json`), and the repeat=7 checked-in
  gate passes at p50/p95/max total `2514/2953/2953us`
  (`target/fret-diag/perf-code-editor-syntax-prefetch-gate/1778392286801/bundle.schema2.json`).
- [ ] Prototype row/fragment replay for `ui-gallery-code-editor-torture-autoscroll-steady` so unchanged rows do not
  repaint the whole visible canvas every frame; keep the dedicated Windows baseline as the guardrail.
- [x] Add percentiles (p50/p95) for `diag stats` bundle summaries (typical perf review).
- [x] Export phase sub-events in `trace.chrome.json` derived from `debug.stats.*_time_us`.
- [x] Export per-run frame percentiles into `check.perf_thresholds.json` for quick scanning.
- [x] Add miss-only renderer spans for pipeline creation and intermediate target allocation/eviction.
- [x] Make “phase timeline” → “top hotspots” linkage explicit in one place (docs + tool output).
  - `diag triage` now emits `phase.timeline_hotspots`, which ties the worst-frame phase times to the
    layout, scroll, paint, and renderer hotspot examples in the same summary.

## Windows-specific

- [x] Document PIX capture steps for `fret-ui-gallery.exe` (GPU-side sanity when CPU looks good).
- [x] Document ETW/WPR profile preset to correlate spikes with OS scheduling/IO.

## Known issues / stability

- [ ] Investigate occasional `thread 'main' has overflowed its stack` on `fret-ui-gallery.exe` exit after long perf suites.
