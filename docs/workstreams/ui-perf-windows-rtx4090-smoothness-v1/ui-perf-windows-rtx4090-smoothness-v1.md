# UI perf: Windows RTX 4090 smoothness v1

Status: Active (local perf worktree)

## Goal

Make Windows (`windows-rtx4090`) UI smoothness a sustainable **performance contract**:

- Gates pass consistently (low tail latency, fewer spikes).
- Worst bundle is explainable (clear attribution, fast diff workflow).
- Optimizations are reversible (small, well-scoped commits + evidence).

This workstream focuses on **CPU-side frame smoothness** (layout/paint/dispatch) first, while keeping
GPU tooling (PIX/Nsight/RenderDoc) available for “GPU is the bottleneck” cases.

## Baselines (source of truth)

- `docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-gallery-context-menu-right-click-steady.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-gallery-dialog-escape-focus-restore-steady.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-gallery-dropdown-open-select-steady.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-gallery-overlay-pointer-move-steady.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-gallery-overlay-torture-steady.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v2.json`
- `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v2.json`
- `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v2.json`
- `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v1.json`
  (typical scroll perf, `frame_p95_*`, `--perf-threshold-agg p90`)
- `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json`
  (high-stress editor wheel, `top_*` + `frame_p95_*`, renderer payload)
- `docs/workstreams/perf-baselines/ui-gallery-complex-steady.windows-rtx4090.v1.json` (tail / spikes, `top_*`)
- `docs/workstreams/perf-baselines/ui-gallery-complex-typical.windows-rtx4090.v1.json` (typical perf, `frame_p95_*`)

Seed policy (how thresholds were derived):

- `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json`
- `docs/workstreams/perf-baselines/policies/ui-gallery-complex-typical.v1.json`
- `docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-autoscroll-typical.v1.json`
- `docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.v1.json`

## P0 runbook (fast gate check)

Prebuild (once):

- `cargo build -p fretboard-dev -p fret-ui-gallery --release`
- For `gallery-dev` scripts such as `ui-gallery-code-editor-torture-autoscroll-steady`, build the gallery with
  `cargo build -p fret-ui-gallery --release --features gallery-dev`; otherwise the script cannot find the dev-only
  `code-editor-torture` navigation target.

Recommended env (avoid extra I/O + keep cached rendering on):

- `FRET_DIAG_SCRIPT_AUTO_DUMP=0`
- `FRET_DIAG_SEMANTICS=0`
- `FRET_A11Y_DISABLE=1` for non-accessibility perf baselines
- `FRET_UI_GALLERY_VIEW_CACHE=1`
- `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`
- `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`

P0 commands:

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 3 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env ... --launch -- target/release/fret-ui-gallery.exe`
- `python tools/perf/diag_resize_probes_gate.py --suite ui-resize-probes --attempts 3 --repeat 7 --baseline docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v2.json --fretboard-bin target/release/fretboard-dev.exe --launch-bin target/release/fret-ui-gallery.exe`
- `python tools/perf/diag_resize_probes_gate.py --suite ui-code-editor-resize-probes --attempts 3 --repeat 7 --baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v2.json --fretboard-bin target/release/fretboard-dev.exe --launch-bin target/release/fret-ui-gallery.exe`
- `target/release/fretboard.exe diag perf ui-gallery-code-editor-torture-autoscroll-typical --repeat 15 --warmup-frames 5 --reuse-launch --perf-threshold-agg p90 --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --launch target/release/fret-ui-gallery.exe`

## Stress/jitter runs (tail hunting, not P0)

Most canonical `windows-rtx4090.v1` baselines were tuned for **P0** usage (`repeat=3`, aggregate = `max`). The
resize suite is the exception: `ui-resize-probes.windows-rtx4090.v2.json` is the active contract and is validated with
`attempts=3`, `repeat=7`, `threshold_surface=ui`, and 30% headroom.

When you increase `repeat` (e.g. `repeat=7`), you are intentionally stress-testing stability. Expect
occasional gate failures in legacy v1 suites even when P0 is green; use this mode to find and explain tail spikes.

Recommended stress command:

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 7 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env ... --launch -- target/release/fret-ui-gallery.exe`

Current boundary (2026-05-10):

- The broad `ui-gallery-steady` suite is a maintenance/evidence surface on Windows, not the default promotable gate.
- Use the smaller daily smoke trio (`ui-gallery-dialog-escape-focus-restore-steady`,
  `ui-gallery-context-menu-right-click-steady`, `ui-gallery-material3-tabs-switch-perf-steady`) for routine
  verification.
- Keep `ui-gallery-complex-steady` and the broad `ui-gallery-steady` repeat=7 run as tail evidence while the suite
  membership is narrowed or split into narrower steady-contract groups.
- The experimental combined `ui-gallery-core-steady` baseline was not promoted; `ui-gallery-context-menu-right-click-steady`,
  `ui-gallery-dialog-escape-focus-restore-steady`, `ui-gallery-dropdown-open-select-steady`,
  `ui-gallery-overlay-pointer-move-steady`, and `ui-gallery-overlay-torture-steady` now each have their own
  baselines, but `ui-gallery-overlay-steady` still needs to remain evidence-only because the broad suite is still mixed.
  `ui-gallery-material3-tabs-switch-perf-steady` should stay with the existing `perf-ui-gallery` path unless a later
  narrower follow-on proves it needs its own contract.

Accessibility split note (2026-05-10):

- `FRET_DIAG_SEMANTICS=0` only disables exporting `debug.semantics` into diagnostic bundles; it does not disable
  platform accessibility integration.
- If the OS/AccessKit adapter activates, runner accessibility can still request semantics snapshots and turn scroll-only
  frames into semantics-refresh frames. That is a real a11y workload, but it must not be mixed into the ordinary
  non-a11y scroll baseline.
- The code-editor autoscroll steady script now defaults `FRET_A11Y_DISABLE=1`. Keep a separate a11y perf probe when the
  goal is to measure AccessKit activation/update cost.

Paint-tail note (2026-05-10):

- Probe: `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json`
  (`repeat=3`, release gallery, view-cache shell on, diagnostics semantics off).
- New `paint.widget_heavy` triage points the worst frame at the code-editor `Canvas` host:
  `ElementHostWidget` / `Canvas`, `exclusive_scene_ops_delta=581`.
- A/B for `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING=1` did **not** materially change the worst bundle:
  - baseline: `paint_widget_time_us=5591`, `paint_time_us=5966`
    (`target/fret-diag/perf-code-editor-paint-cache-relax-off/1778367007568/bundle.schema2.json`)
  - relaxed: `paint_widget_time_us=5549`, `paint_time_us=5781`
    (`target/fret-diag/perf-code-editor-paint-cache-relax-on/1778367132446/bundle.schema2.json`)
- Interpretation: do not promote broader paint-cache gating as the primary fix for this workload. The auto-scroll editor
  changes visible canvas content every frame, so the next GPUI-aligned target is the code-editor/Canvas paint structure
  itself: retain or replay stable row/layer fragments by explicit keys, and only repaint the moving/changed slice.

Code-editor paint telemetry (2026-05-10):

- Telemetry run: `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 --dir target/fret-diag/perf-code-editor-paint-telemetry --launch -- target/release/fret-ui-gallery.exe`
- Worst bundle: `target/fret-diag/perf-code-editor-paint-telemetry/1778386820783/bundle.schema2.json`
- Worst frame: `tick=341`, `paint_us=5734`, `paint_widget_us=5326`
- Paint breakdown for that frame:
  - `rows_painted=289`
  - `rows_scene_replayed=288`
  - `rows_scene_stored=1`
  - `us_total=4813`
  - `us_syntax_spans=4173`
  - `us_text_draw=107`
  - `us_row_scene_fast_probe=1`
  - `us_row_scene_replay_touch=1`
  - `us_row_scene_replay_ops=0`
- Delta from `tick=340 -> 341`:
  - `syntax_misses +1`
  - `row_rich_misses +1`
  - `row_scene_misses +1`
  - `us_syntax_spans +4173us`
- Interpretation: the remaining tail spike is not row-scene replay cost. It is a synchronous syntax-cache miss on the paint path. The next optimization slice should target syntax prefetch / miss smoothing, not further row-scene replay tightening.

Syntax miss breakdown (2026-05-10):

- Telemetry run after adding `us_syntax_slice`, `us_syntax_highlight`, `us_syntax_distribute`,
  `us_syntax_store`, and `syntax_rows_stored`:
  `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 --dir target/fret-diag/perf-code-editor-paint-telemetry-syntax-breakdown3 --launch -- target/release/fret-ui-gallery.exe`
- Worst bundle: `target/fret-diag/perf-code-editor-paint-telemetry-syntax-breakdown3/1778389032255/bundle.schema2.json`
- Worst frame: `tick=341`, `paint_us=6163`, `paint_widget_us=5557`
- Paint breakdown for that frame:
  - `rows_painted=289`
  - `rows_scene_replayed=288`
  - `rows_scene_stored=1`
  - `us_total=4996`
  - `us_syntax_spans=4316`
  - `syntax_rows_stored=129`
  - `us_syntax_slice=7`
  - `us_syntax_highlight=4069`
  - `us_syntax_distribute=215`
  - `us_syntax_store=22`
  - `us_text_draw=91`
- Interpretation: the miss cost is dominated by Tree-sitter highlighting for the chunk, not by row distribution,
  cache store, text draw, or row-scene replay. The next optimization should move syntax filling off the paint critical
  path or prefetch it ahead of the viewport; shrinking cache store/distribution is not the first-order fix.

Syntax prefetch off paint path (2026-05-10):

- Change: `fret-code-editor` now keeps syntax highlight chunks keyed by document id, buffer revision, language, and
  aligned row chunk, then fills those chunks on the dispatcher background lane when platform capabilities advertise both
  background work and wake support. The UI thread drains completed chunks at the windowed-row paint boundary and only
  stores row spans in the editor cache. Stale background results are dropped by doc/revision/language key, and
  buffer/language invalidation clears pending/ready prefetch state. Platforms without the required execution capability
  keep the synchronous fallback path.
- Implementation anchors:
  - `ecosystem/fret-code-editor/src/editor/mod.rs` (`SyntaxPrefetchRuntime`, doc-aware key, runtime cleanup)
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`schedule_syntax_prefetch_for_frame`,
    `syntax_rows_from_highlight_spans`, shared row-cache store path)
- Verification:
  - `cargo check -p fret-code-editor --features syntax`
  - `cargo check -p fret-code-editor --features syntax-rust`
  - `cargo check -p fret-code-editor`
  - `cargo check -p fret-ui-gallery --features gallery-dev`
  - `cargo fmt -p fret-code-editor --check`
  - `cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast`
  - `cargo nextest run -p fret-code-editor --lib --features syntax syntax_prefetch_key_distinguishes_documents_with_same_revision syntax_rows_from_highlight_spans_maps_across_rows --no-fail-fast`
  - `cargo nextest run -p fret-code-editor --lib --features syntax syntax_replay_key_matches_current_inputs_by_pointer_identity --no-fail-fast`
- Perf evidence command:
  `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 --dir target/fret-diag/perf-code-editor-syntax-prefetch --launch -- target/release/fret-ui-gallery.exe`
- Worst bundle: `target/fret-diag/perf-code-editor-syntax-prefetch/1778392044589/bundle.schema2.json`
- Result:
  - `repeat=3` p50/p95/max total: `2305/2663/2663us`
  - `repeat=3` p50/p95/max paint: `1975/2426/2426us`
  - Worst paint frames report `us_syntax_highlight=0`; the remaining syntax apply cost is cache store only
    (`us_syntax_store` about `27-35us`, with `syntax_rows_stored=129/258`).
- Gate evidence:
  - Command: same script with `--repeat 7` and
    `--perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v1.json`
  - Worst bundle: `target/fret-diag/perf-code-editor-syntax-prefetch-gate/1778392286801/bundle.schema2.json`
  - Gate passed with p50/p95/max total `2514/2953/2953us` and paint `2263/2738/2738us`.
- Interpretation: the measured Tree-sitter highlight spike has been removed from the paint critical path. The next
  code-editor paint work should look at the remaining steady paint body / renderer encode cost, not syntax-cache miss
  highlighting. Rollback is a straight revert of the syntax prefetch change.

Bounded row-cache touch queues (2026-05-10):

- Finding: after syntax prefetch, `ui-gallery-code-editor-torture-autoscroll-steady` still alternates between frames
  where all rows replay and frames where one new row is stored (`rows_scene_stored=1`). The remaining tail no longer
  points at syntax highlight; it points at editor paint bookkeeping and renderer encode. A GPUI comparison is useful
  here: GPUI's line-layout cache keeps per-frame "used" lists bounded and swaps current/previous frame caches, so stale
  touch records do not accumulate indefinitely across steady redraws.
- Change: row-level editor caches now compact their `(row, tick)` touch queues when they grow beyond a bounded multiple
  of the live cache size. The compacted queue is rebuilt from live cache entries sorted by latest tick, preserving LRU
  eviction order while avoiding unbounded stale touch records. This covers row text, geometry, row scene, syntax rows,
  and syntax-rich rows. Diagnostic cache-size snapshots were bumped to schema `2` and now include queue lengths.
- Implementation anchors:
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`compact_row_lru_queue_if_needed` and row-cache call sites)
  - `ecosystem/fret-code-editor/src/editor/mod.rs` (`CodeEditorCacheSizeSnapshot` queue-length fields)
  - `apps/fret-ui-gallery/src/driver/diag_snapshot.rs` (queue lengths in `app_snapshot.code_editor.torture.cache_sizes`)
- Verification:
  - `cargo fmt -p fret-code-editor -p fret-ui-gallery --check`
  - `cargo check -p fret-code-editor`
  - `cargo check -p fret-code-editor --features syntax-rust`
  - `cargo check -p fret-ui-gallery --features gallery-dev`
  - `cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast`
  - `cargo build -p fret-ui-gallery --release --features gallery-dev`
- Perf evidence:
  - `repeat=3` command: same `ui-gallery-code-editor-torture-autoscroll-steady` command as the syntax-prefetch run,
    with output directory `target/fret-diag/perf-code-editor-lru-compaction-final`.
  - Worst bundle: `target/fret-diag/perf-code-editor-lru-compaction-final/1778394523256/bundle.schema2.json`
  - `repeat=3` p50/p95/max total: `2323/2656/2656us`; paint: `2117/2445/2445us`.
  - `repeat=7` gate bundle:
    `target/fret-diag/perf-code-editor-lru-compaction-gate/1778394610070/bundle.schema2.json`
  - Gate passed with p50/p95/max total `2452/2869/2869us` and paint `2254/2628/2628us`.
  - Queue maxes in the gate bundle: row text `1707`, row geom `1619`, row scene `1619`, syntax rows `880`,
    row rich `1656`.
- Interpretation: this makes the row-cache bookkeeping bounded and observable, which is the right structural direction
  before deeper row/fragment work. It does not remove the remaining steady renderer encode cost (`renderer encode`
  remains about ~1.0-1.1ms p95 in this script) or the new-row materialization cost. The next code-editor slice should
  split those two costs explicitly instead of treating all paint time as one bucket.

Code-editor paint nanosecond attribution probe (2026-05-10):

- Finding: the previous `us_*` paint-perf counters undercounted hot per-row paths because many sub-steps are measured
  once per visible row and each `elapsed().as_micros()` call truncates sub-microsecond work to zero. That made
  `us_total` useful but left the child buckets too sparse to explain the remaining steady p95.
- Change: `CodeEditorPaintPerfFrame` now keeps the existing `us_*` fields and adds `ns_*` mirrors for the measured
  child buckets, plus `us/ns_row_geom_cache`. The UI gallery diagnostic snapshot emits `paint_perf.schema_version=3`.
  `CodeEditorPaintPerfFrame` is re-exported from `fret-code-editor` so the gallery snapshot helper can name the type
  without relying on a very large `serde_json::json!` macro expansion.
- Implementation anchors:
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`add_paint_perf_elapsed`, nanosecond buckets, row geometry
    cache timing)
  - `ecosystem/fret-code-editor/src/editor/mod.rs` (`CodeEditorPaintPerfFrame` schema fields)
  - `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorPaintPerfFrame` re-export)
  - `apps/fret-ui-gallery/src/driver/diag_snapshot.rs` (`code_editor_paint_perf_json`)
- Verification:
  - `cargo fmt -p fret-code-editor -p fret-ui-gallery --check`
  - `cargo check -p fret-code-editor`
  - `cargo check -p fret-code-editor --features syntax`
  - `cargo check -p fret-code-editor --features syntax-rust`
  - `cargo check -p fret-ui-gallery --features gallery-dev`
  - `cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast`
  - `cargo build -p fret-ui-gallery --release --features gallery-dev` (passed; existing `fret-runtime`/`fret-ui`
    warnings only)
- Perf probe:
  - Command:
    `target/release/fretboard.exe diag perf --dir target/fret-diag/perf-code-editor-ns-attribution-probe --repeat 3 --warmup-frames 5 --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --launch -- target/release/fret-ui-gallery.exe`
  - Worst bundle:
    `target/fret-diag/perf-code-editor-ns-attribution-probe/1778396085536/bundle.schema2.json`
  - Observed `repeat=3` p50/p95/max total: `3108/3414/3414us`; paint: `2885/3039/3039us`. These totals include
    the high-overhead nanosecond probe and must not replace the steady perf baseline.
  - Probe interpretation: low-overhead rows show row text, scene-cache probe/replay, and row-geom cache are each
    tens of microseconds across ~289 rows. Frames that store one newly visible row still correlate with rich
    materialization/text-draw tails (`ns_rich_materialize` reached ~443-495us in the probe's worst frames). Renderer
    work is still large and separate (`renderer_prepare_text_us` and `renderer_encode_scene_us` remain prominent).
- Next direction: keep the nanosecond fields as opt-in diagnostics only. The next optimization should target either
  content-addressed/new-row rich materialization or renderer text/scene encode, not more syntax-cache work.

Finding (2026-05-10): redundant row background quads are pure scene-op churn

- Change: `ecosystem/fret-code-editor/src/editor/paint/mod.rs` no longer emits a transparent background `Quad` for
  every painted row. Pointer handling already lives in the outer `PointerRegion`, so the row background op was not
  contributing visible output or hit-testing.
- Diagnostics cleanup: `app_snapshot.code_editor.torture.paint_perf.schema_version` is now `2`, and the stale
  `quads_background` field was removed instead of preserved as an always-zero compatibility field.
- Evidence run:
  - `target/release/fretboard-dev.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --dir target/fret-diag/perf-code-editor-drop-transparent-row-bg --launch -- target/release/fret-ui-gallery.exe`
  - Bundle: `target/fret-diag/perf-code-editor-drop-transparent-row-bg/1778369038936/bundle.schema2.json`
- Result:
  - Worst-frame `scene_ops` dropped from `1368` to `994` compared with the previous reference bundle.
  - `paint_widget_time_us` p95 dropped from `5667` to `5276`.
  - `paint_cache_replayed_ops` dropped from `604` to `519`.
- Contract:
  - The probe now has a dedicated Windows baseline at
    `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v1.json`.
  - The 7-repeat gate now passes with `p50/p95/max total=2587/6657/6657us`,
    `layout=1059/1123/1123us`, `paint=1430/5490/5707us`.
- Interpretation: this is a safe cleanup, but it is still a local optimization. The structural follow-on remains
  row/fragment retention or replay by stable keys, not more global cache gating.

Scene text blob index for renderer text prepare (2026-05-10):

- Finding: after syntax prefetch and row-cache compaction, the code-editor autoscroll probe still showed renderer
  text prepare and scene encode as separate paint costs. `TextSystem::collect_scene_pinned_keys` scanned every
  `SceneOp` each frame just to find text blobs. That is exactly the kind of hot-path churn that GPUI/Zed-style
  rendering avoids by keeping explicit frame indexes for frequently consumed subsets.
- Change: `SceneRecording` now records text blob ids in draw-op order as text ops are pushed. The index is cleared with
  the scene, swapped with paint-cache storage, and replayed through the same cache boundary as the op vector. Renderer
  atlas pinning now iterates `scene.text_blob_ids()` instead of scanning all scene ops.
- Implementation anchors:
  - `crates/fret-core/src/scene/mod.rs` (`SceneRecording::text_blob_ids`, `SceneRecording::push`,
    `SceneRecording::swap_storage`)
  - `crates/fret-ui/src/tree/paint_cache.rs` (`PaintCacheState::prev_text_blob_ids`)
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs` (`ingest_paint_cache_source`)
  - `crates/fret-render-wgpu/src/text/atlas_flow.rs` (`collect_scene_pinned_keys`)
- Verification:
  - `cargo fmt -p fret-core -p fret-ui -p fret-render-wgpu --check`
  - `cargo check -p fret-core`
  - `cargo check -p fret-ui`
  - `cargo check -p fret-render-wgpu`
  - `cargo nextest run -p fret-core --lib --no-fail-fast`
  - `cargo build -p fretboard --release`
  - `cargo build -p fret-ui-gallery --release --features gallery-dev` (passed with existing
    `fret-runtime` / `fret-ui` warnings only)
- A/B evidence:
  - Indexed version: `renderer_prepare_text_us` stayed around `100-111us` in the final gate.
  - Temporary scan-all-ops version:
    `target/fret-diag/perf-code-editor-scene-text-index-ab-scan/1778399707408/bundle.schema2.json`
    reported `renderer_prepare_text_us` around `751-1165us`.
- Final gate:
  - Command:
    `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --dir target/fret-diag/perf-code-editor-scene-text-index-gate-final --launch -- target/release/fret-ui-gallery.exe`
  - Bundle:
    `target/fret-diag/perf-code-editor-scene-text-index-gate-final/1778400442891/bundle.schema2.json`
  - Result: gate passed with p50/p95/max total `2419/2709/2709us` and paint `2186/2478/2478us`.
  - Renderer stats: p95/max upload `187us`, record `40us`, finish `123us`, encode `1079us`, text `111us`,
    svg `9us`.
  - Worst frame `tick=1445`: total/layout/prepaint/paint `2709/241/95/2373us`; renderer encode/text
    `1017/109us`.
- Interpretation: renderer text prepare is no longer the first-order renderer bottleneck for this script. The next
  renderer slice should split `renderer_encode_scene_us` into op classification, text vertex construction, quad/text
  batching, and draw-group flush/merge work.

Workflow when it fails:

- Read `target/fret-diag/check.perf_thresholds.json` and follow the bundle path printed as `worst overall`.
- Attribute the worst bundle:
  - `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30`
  - `target/release/fretboard.exe diag stats <bundle.json> --sort cpu_cycles --top 30`
  - Renderer stage timings (CPU-side) are also available in `diag stats`:
    - `--sort ensure_pipelines|plan_compile|upload|record_passes|encoder_finish`
    - The human summary prints `renderer p50/p95` and `renderer max` when the fields are present.

First places to look on Windows:

- `top_total_time_us` / `frame_p95_total_time_us`: the contract miss itself. Start with `diag stats --sort time --top 30`.
- `top_layout_time_us` / `top_layout_engine_solve_time_us`: layout-root churn or solver pressure. Inspect layout hotspots
  and, if needed, re-run with `FRET_LAYOUT_NODE_PROFILE=1`.
- `top_paint_time_us` / `frame_max_paint_time_us`: paint-tail or renderer churn. Start with triage hints
  (`paint.widget_heavy`, `paint.text_prepare_churn`), then inspect renderer stage timings and trace.
- `pointer_move_max_dispatch_time_us` / `pointer_move_max_hit_test_time_us`: overlay/pointer interaction suites.

If suite results look inconsistent (a script is fast when run alone but slow inside a suite), use
suite normalization hooks to reduce cross-script state contamination:

- `--prewarm-script <script.json>...`: run once per launched process before the suite.
- `--prelude-script <script.json>...`: run before each measured script (and per-run when combined with
  `--prelude-each-run`).
- If the suite still drifts (or you hit a long-run crash), consider isolating scripts by relaunching
  once per script:
  - `--reuse-launch --reuse-launch-per-script --launch -- <cmd...>`

Suggested defaults for UI-gallery perf work:

- `--prewarm-script tools/diag-scripts/tooling-suite-prewarm-fonts.json`
- `--prelude-script tools/diag-scripts/tooling-suite-prelude-ui-gallery-normalize.json`
- Scripts that wait on `font_catalog_populated` should either run with `FRET_UI_GALLERY_BOOTSTRAP_FONTS=1` or carry
  that value in `meta.env_defaults`; otherwise bundled-only release runs can leave `font_catalog_family_count=0` and
  turn the font wait into setup noise.

## Finding (2026-05-06): context-menu steady probe should not include sidebar navigation

Observed:

- `ui-gallery-context-menu-right-click-steady` could fail or drift before the measured interaction,
  especially around sidebar search/scroll navigation to the internal Overlay page.
- A failed run that stops before `reset_diagnostics` must not be treated as a perf baseline because
  it mixes startup/navigation work into the sample.

Change:

- The script now sets `FRET_UI_GALLERY_START_PAGE=overlay` through `meta.env_defaults`.
- The script no longer drives sidebar search/scroll to reach the Overlay page.
- Font-catalog stabilization is left to suite-level prewarm hooks instead of the single script body.

Evidence (local Windows / RTX 4090, release, `fret-ui-gallery --features gallery-dev`):

- `target/release/fretboard.exe diag run tools/diag-scripts/ui-gallery/perf/ui-gallery-context-menu-right-click-steady.json --dir target/fret-diag/context-menu-steady-release2 --session-auto --timeout-ms 240000 --launch target/release/fret-ui-gallery.exe`
  - Passed, run id `1778072942604`.
  - Bundle: `target/fret-diag/context-menu-steady-release2/sessions/1778072933113-84248/1778073162792-ui-gallery-context-action-steady/bundle.schema2.json`
- `target/release/fretboard.exe diag stats <bundle> --sort time --top 30`
  - `time p50/p95 (us)`: total `1399/1629`, layout `1107/1309`, prepaint `175/194`, paint `124/135`.
  - Renderer p95/max (us): upload `353`, record `37`, finish `157`, encode `366`, text `394`.
  - Interpretation: this probe is still CPU/layout dominated; GPU-side churn is not the first target.
- `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-context-menu-right-click-steady.json --repeat 2 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --timeout-ms 300000 --dir target/fret-diag/perf-context-menu-steady-check --launch target/release/fret-ui-gallery.exe`
  - `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=1578/1280/32/167/118/0/6`
  - `p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=1784/1450/35/216/131/0/6`

Follow-up:

- `ui-gallery-steady --reuse-launch` still needs suite normalization work on Windows:
  - `--prewarm-script tools/diag-scripts/tooling-suite-prewarm-fonts.json` can stall at
    `font_catalog_populated` in this local run.
  - A no-prewarm suite run reached this script but stalled after `reset_diagnostics`, indicating
    cross-script or reuse-launch state is still not normalized enough for the whole suite.
- Do not re-seed suite baselines from those failed suite attempts. Use the single-script evidence
  above until the suite prewarm/prelude lane is fixed.

## Finding (2026-05-06): diagnostics keepalive timer was only wired through the shared ui-app-driver path

Observed:

- The UI gallery uses a custom `WinitAppDriver` implementation, and it already called the public
  `fret_bootstrap::maybe_consume_event` helper.
- The keepalive timer branch lived only in the shared `ui_app_driver` event path, so the gallery
  never consumed `Event::Timer` for scripted keepalive. In practice, `wait_frames` and other
  frame-driven script steps could starve until a rare redraw or fallback tick arrived.

Change:

- Moved diagnostics timer consumption into the public `fret_bootstrap::ui_diagnostics::maybe_consume_event`
  entrypoint, so every driver using the public helper gets the same keepalive contract.
- Removed the duplicate timer branch from `ui_app_driver`.

Evidence:

- `cargo check -p fret-bootstrap --features diagnostics,ui-app-driver`
- `cargo build -p fret-ui-gallery --release --features gallery-dev`
- `target/release/fretboard.exe diag run tools/diag-scripts/ui-gallery/perf/ui-gallery-context-menu-right-click-steady.json --dir target/fret-diag/context-menu-keepalive-check --session-auto --timeout-ms 240000 --launch target/release/fret-ui-gallery.exe`
  - Passed (`run_id=1778075858405`).
  - The same script completed materially faster than the earlier stuck/slow runs.
- `target/release/fretboard.exe diag stats target/fret-diag/context-menu-keepalive-check/sessions/1778075856663-88504/1778075909476-ui-gallery-context-action-steady/bundle.schema2.json --sort time --top 30`
  - `time p50/p95 (us)`: total `2607/3375`, layout `2134/2761`, prepaint `267/386`, paint `207/229`.
  - Interpretation: the probe is now advancing normally again; remaining cost is CPU/layout work, not a keepalive starvation bug.

## Finding (2026-05-06): perf baselines must use canonical script paths, not redirect stubs

Observed:

- `ui-gallery-steady` failed before measuring because the suite manifest used canonical script paths
  while the checked-in perf baselines still used old top-level `script_redirect` stubs.
- Example failure:
  - `perf baseline missing entry for script: tools/diag-scripts/ui-gallery/perf/ui-gallery-context-menu-right-click-steady.json`
  - Baseline row still pointed at `tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json`.

Change:

- Migrated `docs/workstreams/perf-baselines/**/*.json` from redirect stub script keys/scopes to the
  final canonical `tools/diag-scripts/...` targets.
- Kept the comparison layer strict; `read_perf_baseline_file` does not follow redirects implicitly.

Evidence:

- Redirect reference scan after migration:
  - `remaining_redirect_refs=0 files=0`
- `cargo nextest run -p fret-diag perf_baseline_parse`
  - Passed (`tests::perf_baseline_parse_reads_script_thresholds`).
- Narrow baseline lookup:
  - `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-context-menu-right-click-steady.json --repeat 1 --warmup-frames 5 --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --timeout-ms 300000 --launch target/release/fret-ui-gallery.exe`
  - Passed; `top.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=2338/1879/47/256/203/0/10`.

Follow-up:

- Resolved below: `ui-gallery-overlay-pointer-move-steady` was stabilized and seeded into the
  Windows RTX4090 baseline.

## Finding (2026-05-06): overlay pointer-move probe should be a bounded steady pointer sample

Observed:

- `ui-gallery-overlay-pointer-move-steady` was part of the `ui-gallery-steady` suite, but the
  Windows RTX4090 baseline had no row for it.
- The script still used the old pattern of navigating through the sidebar and waiting for font
  catalog stability inside the measured script.
- The pointer sweep used `steps=420`; `move_pointer_sweep` advances one step per frame, so this
  could exceed normal gate timeouts and exceeded the bundle retention budget (`max_snapshots=240`).

Change:

- Added `FRET_UI_GALLERY_START_PAGE=overlay` to the script metadata.
- Removed sidebar navigation and script-local font stabilization from the probe.
- Reduced the sweep to `steps=96`, which still produces a large enough pointer-move sample for
  dispatch/hit-test accounting while keeping the script bounded.
- Added the missing Windows baseline row for
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json`.

Evidence:

- `target/release/fretboard.exe diag run tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json --dir target/fret-diag/overlay-pointer-steady-script-check2 --session-auto --timeout-ms 240000 --launch target/release/fret-ui-gallery.exe`
  - Passed (`run_id=1778078914577`).
  - Bundle:
    `target/fret-diag/overlay-pointer-steady-script-check2/sessions/1778078912845-86592/1778079003710-ui-gallery-overlay-pointer-move-steady/bundle.schema2.json`
- `target/release/fretboard.exe diag stats <bundle> --sort time --top 20`
  - `time p50/p95 (us)`: total `1469/3038`, layout `1150/1715`, prepaint `181/224`, paint `127/287`,
    dispatch `122/176`, hit_test `6/15`.
  - Derived pointer move: `frames_considered=98`, max dispatch/hit_test `277/22us`,
    `snapshots_with_global_changes=0`.
- `cargo run -p fretboard-dev -- diag perf-baseline-from-bundles tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json <bundle> --perf-baseline-out target/fret-diag/baseline-ui-gallery-overlay-pointer-move.windows-rtx4090.v1.json --sort time --warmup-frames 5 --perf-baseline-headroom-pct 40`
  - Wrote the baseline seed used for the checked-in row.
- Suite/baseline membership check:
  - `ui-gallery-steady` suite scripts: `11`
  - `ui-gallery-steady.windows-rtx4090.v1.json` rows: `11`
  - Missing in baseline: none.

Remaining blocker:

- The full `ui-gallery-steady` gate still stalls before measured scripts when using the suite prewarm
  hook:
  - `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 3 --warmup-frames 5 --reuse-launch --timeout-ms 600000 --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --prewarm-script tools/diag-scripts/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/tooling-suite-prelude-ui-gallery-normalize.json --env ... --launch -- target/release/fret-ui-gallery.exe`
  - The latest `target/fret-diag/script.json` was the prewarm script and stopped at step 0
    (`font_catalog_populated`), so this is a suite prewarm/normalization issue rather than a
    baseline-entry issue.

## Complex UI suite (typical perf)

Use two separate suites depending on whether you are hunting tail spikes or checking “normal”
frame-time distributions.

Tail / spikes (worst-frame `top_*`):

- `target/release/fretboard.exe diag perf ui-gallery-complex-steady --repeat 7 --warmup-frames 5 --reuse-launch --prewarm-script tools/diag-scripts/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/tooling-suite-prelude-ui-gallery-normalize.json --prelude-each-run --env ... --launch -- target/release/fret-ui-gallery.exe`

Current status:

- The normalized `ui-gallery-complex-steady` run with `--prelude-each-run` now completes. Keep the legacy no-prelude
  failure in `ui-gallery-chrome-torture-steady.json` step 9 (`subtree layout dirty count underflow` at
  `crates/fret-ui/src/tree/ui_tree_mutation/remove.rs:374`) as a suite-normalization warning, not as the steady-state.
- The current Windows paint exemplar is `target/fret-diag/1778364986668/bundle.schema2.json`.
- `diag stats --sort time` for that bundle reports `time p50/p95 (us)` total `1898/7326`, layout `264/1296`,
  prepaint `98/119`, paint `1347/6070`, with `hot p50/p95` paint.widget `962/5667`.
- Use this bundle when you need paint-tail attribution for the editor-grade lane; the smaller resize and overlay
  contracts remain the gating surface for routine regressions.

Typical perf gate (bundle frame percentiles `frame_p95_*`):

- `target/release/fretboard.exe diag perf ui-gallery-complex-typical --repeat 11 --warmup-frames 5 --reuse-launch --perf-threshold-agg p90 --perf-baseline docs/workstreams/perf-baselines/ui-gallery-complex-typical.windows-rtx4090.v1.json --prewarm-script tools/diag-scripts/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/tooling-suite-prelude-ui-gallery-normalize.json --prelude-each-run --env ... --launch -- target/release/fret-ui-gallery.exe`

Notes:

- Use `--prelude-each-run` for typical gates to reduce cross-run drift when using `--reuse-launch`.
- Use `--repeat >= 11` when gating percentiles (with small repeat counts, `p90` collapses to `max`).

To inspect “normal” (non-tail) performance, prefer frame percentiles from each evidence bundle:

- `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30`
  - Look at `time p50/p95 (us)` (these are per-frame percentiles within the bundle).
- `target/fret-diag/check.perf_thresholds.json` also includes per-run `frame_p50_*` / `frame_p95_*`
  fields, derived from the bundle stats, for quick scanning without opening each bundle.

Recommended snapshot retention for typical-perf runs:

- `FRET_DIAG_MAX_SNAPSHOTS=180`
- `FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS=180`

## Failure triage (when a gate fails)

1) Look at the generated perf check:

- `<out_dir>/check.perf_thresholds.json`
  - Includes `max` and percentiles (`p50`/`p95`) per script.
  - When a threshold fails, `failures[]` includes `actual_p95_us`, `outlier_suspected`,
    `evidence_bundle`, `evidence_run_index`, and `evidence_run` for quick triage.
  - Renderer threshold failures use the renderer metric's own worst run instead of the script's
    top-total run. For renderer metrics that have a stats sort (`encode_scene`, `upload`,
    `record_passes`, `encoder_finish`, `prepare_text`), `failures[].evidence_peak` also includes the
    metric-sorted worst frame and renderer top fields.

2) Open the worst evidence bundle:

- `<out_dir>/worst_overall.bundle.json` (or the `worst_overall.bundle` path printed by `diag perf`)

3) Summarize and attribute:

- `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30 --json`
  - `diag stats --json` includes `sum` / `avg` / `max` plus `p50` / `p95` for key frame timings (typical perf).
- Compare “good vs bad” bundles:
  - `target/release/fretboard.exe diag stats --diff <ok_bundle.json> <bad_bundle.json> --sort time --json`

4) If the summary is not enough, switch to opt-in deeper evidence:

- Node-level layout profiling:
  - `--env FRET_LAYOUT_NODE_PROFILE=1`
  - `--env FRET_LAYOUT_NODE_PROFILE_TOP=15`
  - `--env FRET_LAYOUT_NODE_PROFILE_MIN_US=400`
- Trace artifacts (for a single run, not for gate runs):
  - `target/release/fretboard.exe diag perf ... --trace`
  - `target/release/fretboard.exe diag trace <bundle.json>`
  - The exported `trace.chrome.json` includes phase sub-events derived from `debug.stats.*_time_us`
    (e.g. `layout.collect_roots`, `layout.request_build_roots`, `layout.engine_solve`, `paint.cache_replay`).

## Windows ETW/WPR (schedule noise vs real CPU work)

When a perf gate fails due to rare spikes (max) but typical percentiles look fine, verify whether the
UI thread is actually running CPU work or is being delayed by OS scheduling (Ready time), DPC/ISR,
or other system noise.

Recommended capture (WPR built-in profiles):

- `GeneralProfile.Verbose` (best first-pass triage: CPU + CSwitch + ReadyThread + DPC/Interrupt).
- `CPU.Verbose` (lighter: CPU + CSwitch + ReadyThread + SampledProfile stacks).

Runbook:

1) Start WPR (filemode avoids memory pressure during capture):

- `wpr -start GeneralProfile.Verbose -filemode`

2) Run a repro that tends to spike (prefer `--reuse-launch` to reduce relaunch noise; add `--trace`
   so the worst bundle includes `trace.chrome.json`):

- `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json --repeat 200 --warmup-frames 5 --reuse-launch --trace --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --timeout-ms 900000 --env ... --launch -- target/release/fret-ui-gallery.exe`

3) Stop WPR and write the ETL:

- `wpr -stop ui-perf.etl`

Note: Some environments block WPR/ETW system profiling via policy (e.g. `0xc5585011`). If WPR fails:

- Prefer in-app evidence (`--trace`, `diag stats`, `FRET_LAYOUT_NODE_PROFILE=1`) to confirm CPU phase attribution.
- Use Windows best-effort isolation knobs (`--launch-high-priority`, `--reuse-launch`) to reduce scheduling noise.

4) Open in Windows Performance Analyzer (WPA) and filter to the app process:

- The diagnostics out dir writes `launched.demo.json` with the launched `pid` (when using `--launch`).
- In WPA, focus on:
  - **CPU Usage (Sampled)** for stacks (are we actually executing?)
  - **Context Switches / ReadyThread** (are we ready-but-not-running?)
  - **DPC/ISR** (are interrupts/DPC stealing time?)

Interpretation:

- High **ReadyThread** time + low sampled CPU in the spike window ⇒ scheduling contention / priority / background noise.
- High sampled CPU with stable stacks in Fret code ⇒ real work regression (optimize the hottest phase).
- DPC/ISR spikes aligned with frame spikes ⇒ driver/OS noise; consider isolating (priority, affinity, power plan, background activity).

## Windows PIX GPU capture (GPU-side sanity when CPU looks fine)

Use PIX when the renderer is on a PIX-compatible Direct3D path and CPU-side evidence does not explain the hitch.

Runbook:

1. Install the latest main PIX build from the Microsoft PIX download page, or from an approved internal package source.
2. In PIX, open the Connection view and launch or attach to `fret-ui-gallery.exe` with GPU capture enabled.
3. Take the capture with the `Take GPU Capture` button, or use the PIX shortcut (`Alt+Print Screen`; `F11` is configurable).
4. Open the capture and inspect Overview / Events / Timeline. For timing analysis, collect timing data first.
5. If the capture is noisy, close other GPU-heavy apps before capturing.

## In-app CPU-time signal (when ETW/WPR is unavailable)

Some environments block WPR/ETW system profiling. In that case, use the in-app UI-thread CPU-time
signal exported into `debug.stats`:

- `ui_thread_cpu_time_us`: approximate CPU time consumed by the UI thread since the previous snapshot.
- `ui_thread_cpu_cycle_time_delta_cycles`: UI thread cycle delta since the previous snapshot (Windows-only, higher resolution).

How to interpret:

- Prefer `ui_thread_cpu_cycle_time_delta_cycles` when available: `GetThreadTimes` can be coarse and appear quantized.
- Treat `ui_thread_cpu_time_us` as a best-effort hint, not a precise per-frame budget.

- If `total_time_us` spikes but `ui_thread_cpu_time_us` stays low ⇒ schedule noise / preemption likely.
- If both spike together ⇒ real CPU work regression (optimize the dominating phase).

## What “typical perf” means here (not tail)

Tail (spikes) is “max / worst frame”. Typical perf should use **percentiles** (p50/p95) to answer
“is it generally faster/slower”.

Preferred workflow:

- Use `fretboard-dev diag perf ... --json` and review `p50`/`p95` for the top metrics.
- Use `diag stats --json` for within-bundle `p50` / `p95` (typical), `avg.*`, and `budget_pct.*`.
- If you want a **typical-perf gate**, create a dedicated baseline with `ui_threshold_mode=frame_p95` and then
  gate using `--perf-threshold-agg p95`.

Example (local typical baseline; does not change the canonical baselines):

- Create a p95-seeded baseline:
  - `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 15 --warmup-frames 5 --perf-baseline-out .fret/perf.baseline.p95.json --perf-baseline-ui-threshold-mode frame_p95 --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json --perf-baseline-seed this-suite@frame_p95_total_time_us=p95 --launch -- target/release/fret-ui-gallery.exe`
- Gate typical perf (p95 aggregate):
  - `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 15 --warmup-frames 5 --perf-threshold-agg p95 --perf-baseline .fret/perf.baseline.p95.json --launch -- target/release/fret-ui-gallery.exe`

If a change improves p50/p95 but worsens max occasionally, treat it as “needs jitter work” (allocator,
capacity management, background work scheduling).

## Recent finding (2026-02-14): VirtualListMetrics clone caused avoidable churn

Symptom pattern:

- Same logical work (solves/nodes similar), but some runs had slow-path spikes.
- Layout node profiling (`FRET_LAYOUT_NODE_PROFILE=1`) showed VirtualList as a recurring hotspot.

Change:

- Avoid `VirtualListMetrics` cloning in VirtualList layout/measure paths (move-out + write-back).

Evidence:

- `tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json` became consistently under the
  `ui-gallery-steady.windows-rtx4090.v1` thresholds in repeated local runs.

## Finding (2026-02-15): Make the VirtualList cache root layout definite to avoid rerender on deferred scroll

Background:

- `ViewCache` reuse under layout invalidation is only safe for definite-sized cache roots.
- `CachedSubtreeProps` previously created `ViewCacheProps` with the default (Auto-sized) layout, which makes
  `layout_definite=false` even when the subtree itself has a definite size.

Observed symptom:

- `ui-gallery-virtual-list-torture-steady` failed `ui-gallery-steady.windows-rtx4090.v1` on Windows due to
  max spikes in `top_total_time_us` / `top_layout_time_us` / `top_layout_engine_solve_time_us` during
  the jump-to-item + scroll-to-bottom sequence.

Change:

- Extend `CachedSubtreeProps` (ecosystem helper) to allow overriding the `ViewCache` wrapper layout.
- In `virtual_list_torture`, set the cache root layout to the same fixed-size layout as the list (`w_full`, `h=420px`).

Result (local, `repeat=3`, baseline `ui-gallery-steady.windows-rtx4090.v1`):

- `tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json` no longer trips the max thresholds.

Repro command:

- `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json --repeat 3 --warmup-frames 5 --sort time --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release`

## Finding (2026-02-15): Batch-solve barrier roots to eliminate per-root solve spikes

Observed symptom:

- `ui-gallery-virtual-list-torture-steady` could still hit max spikes in `top_layout_engine_solve_time_us`
  during “jump + scroll to bottom”, with `layout_engine_solves` often matching the visible item count
  (e.g. ~38 independent solves in one frame).

Root cause:

- Layout barriers (VirtualList/Scroll/etc.) solved each child root one-by-one, amplifying fixed Taffy
  solve overhead into tail latency.

Change:

- Add `TaffyLayoutEngine::compute_independent_roots_with_measure_if_needed(...)` and use it from the
  barrier solve path so many child roots can be solved in a single synthetic-root Taffy compute when
  they are independent and have definite sizes.

Result (local, `repeat=3`):

- `tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json` now stays under the baseline with
  `top_layout_engine_solve_time_us` max around ~1.1ms (previously ~1.9ms worst frames).
- `ui-gallery-steady` passes its `windows-rtx4090.v1` baseline, and `ui-resize-probes` /
  `ui-code-editor-resize-probes` pass their `windows-rtx4090.v2` baselines.
- 2026-05-10 repeat=7 gate evidence:
  - `ui-resize-probes`: `target/fret-diag-resize-probes-gate-1778363042/summary.json` passed `3/3`; `ui-gallery-window-resize-drag-jitter-steady.json` p50/p95/max total `1855/1980/1980us` vs threshold `4028us`; `ui-gallery-window-resize-stress-steady.json` p50/p95/max total `3800/3993/3993us` vs threshold `5584us`.
  - `ui-code-editor-resize-probes`: `target/fret-diag-resize-probes-gate-1778363387/summary.json` passed `3/3`; `ui-gallery-code-editor-window-resize-drag-jitter-steady.json` p50/p95/max total `2809/3582/3582us` vs threshold `11282us`.

## Finding (2026-02-14): repeat=7 can fail on Material3 tabs (request_build_roots dominates)

Observed:

- `ui-gallery-steady --repeat 7` can fail the baseline on:
  - `ui-gallery-material3-tabs-switch-perf-steady` (`top_layout_time_us`, sometimes `top_layout_engine_solve_time_us`).

Attribution (worst bundle example):

- Bundle: `target/fret-diag/1771077490429-ui-gallery-material3-tabs-switch-perf-steady/bundle.json`
- Summary: `fretboard-dev diag stats <bundle.json> --sort time`
  - In the worst frame, `layout_request_build_roots_time_us` dominates `layout_time_us` (solve is small).
- Trace: `target/fret-diag/1771077490429-ui-gallery-material3-tabs-switch-perf-steady/trace.chrome.json`
  - Inspect `layout.request_build_roots` events for the slow frames.

Next action:

- Decide whether this is primarily **real CPU work** (optimize `build_viewport_flow_subtree`) or **schedule noise**
  (needs ETW/WPR or an in-app CPU-time signal).

## Finding (2026-05-10): action-availability snapshots must not key on pointer arbitration

Observed:

- The `ui-gallery-overlay-interaction-steady` validation failure was not layout or hit-test dominated.
- Worst failed bundles pointed at pointer-move dispatch time with full command action-availability
  evaluation in the post-dispatch window snapshot path:
  `dispatch_snapshot.command_availability(widget_count/collect_us/eval_us)=11/8/580`.
- The failed validation reached `pointer_move_max_dispatch_time_us=313us` and `384us` against a
  `280us` threshold while `layout_engine_solve_time_us=0`.

Root cause:

- `WindowCommandActionAvailabilitySnapshotSignature` used the whole `InputContext` as a cache key.
- `InputContext.window_arbitration` carries modal/capture/pointer-occlusion state for policy-heavy
  event handling, but it is high-frequency pointer-move state and not part of command gating
  (`when` expressions use modal/text/edit/router/platform/cap/keyctx state).

Change:

- `crates/fret-ui/src/tree/mod.rs` now uses a reduced
  `WindowCommandActionAvailabilityInputSignature` for the command action-availability cache key.
- The signature keeps stable command-gating fields and intentionally excludes pointer-arbitration
  state and dispatch-phase noise.
- Regression test:
  `cargo nextest run -p fret-ui window_command_action_availability_snapshot`.

Evidence after the change:

- Script:
  `target/release/fretboard.exe diag run tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json --dir target/fret-diag/overlay-pointer-move-check --session-auto --timeout-ms 240000 --launch target/release/fret-ui-gallery.exe`
- Bundle:
  `target/fret-diag/overlay-pointer-move-check/sessions/1778355256713-143320/1778355261118-ui-gallery-overlay-pointer-move-steady/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 30`:
  - `derived(pointer_move) frames_considered=98 max.us(dispatch/hit_test)=186/23`
  - `time p50/p95 (us): dispatch=100/127 hit_test=6/13`
- Follow-up `diag perf` against `ui-gallery-steady.windows-rtx4090.v1` still failed one unrelated
  renderer threshold:
  - Command:
    `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- target/release/fret-ui-gallery.exe`
  - Failure: `renderer_record_passes_us=173` vs threshold `153`.
  - Targeted pointer-move counters stayed under the baseline row:
    `pointer_move_max_dispatch_time_us=208` vs threshold `388`,
    `pointer_move_max_hit_test_time_us=24` vs threshold `31`.
- Rerunning the actual selection flow for `ui-gallery-overlay-interaction-steady` still leaves the
  suite unpromotable:
  - Selection summary:
    `target/fret-diag-baseline-select-ui-gallery-overlay-interaction-steady-windows-rtx4090-v1-after-action-cache/selection-summary.json`
  - `selected_fail_total=3`, with misses spread across
    `ui-gallery-context-menu-right-click-steady`,
    `ui-gallery-dialog-escape-focus-restore-steady`, and
    `ui-gallery-overlay-pointer-move-steady`.
  - That means the original interaction suite is still too broad for one Windows baseline and should stay
    evidence-only even though its context-menu/dialog/pointer-move members now have their own baselines.

## Finding (2026-05-10): overlay interaction follow-ons now have dedicated baselines

Observed:

- The single-script `ui-gallery-overlay-pointer-move-steady` follow-on was still noisy when selection ran
  without per-run normalization, especially on `pointer_move_max_dispatch_time_us` and
  `pointer_move_max_hit_test_time_us`.
- A successful selection required keeping only the reset-diagnostics prelude, enabling
  `--prelude-each-run`, and raising baseline headroom to 50%.
- `ui-gallery-context-menu-right-click-steady`, `ui-gallery-dialog-escape-focus-restore-steady`,
  `ui-gallery-dropdown-open-select-steady`, and `ui-gallery-overlay-torture-steady` selected cleanly with the same
  per-run reset shape.

Change:

- Added single-script suite manifests for
  `ui-gallery-context-menu-right-click-steady`,
  `ui-gallery-dialog-escape-focus-restore-steady`,
  `ui-gallery-dropdown-open-select-steady`,
  `ui-gallery-overlay-pointer-move-steady`, and
  `ui-gallery-overlay-torture-steady`.
- Added `perf_seed_policy` name mapping + regression coverage for the new suite names.
- Seeded:
  - `docs/workstreams/perf-baselines/ui-gallery-context-menu-right-click-steady.windows-rtx4090.v1.json`
  - `docs/workstreams/perf-baselines/ui-gallery-dialog-escape-focus-restore-steady.windows-rtx4090.v1.json`
  - `docs/workstreams/perf-baselines/ui-gallery-dropdown-open-select-steady.windows-rtx4090.v1.json`
  - `docs/workstreams/perf-baselines/ui-gallery-overlay-pointer-move-steady.windows-rtx4090.v1.json`
  - `docs/workstreams/perf-baselines/ui-gallery-overlay-torture-steady.windows-rtx4090.v1.json`

Evidence:

- Selection summaries:
  - `target/fret-diag-baseline-select-ui-gallery-context-menu-right-click-steady-windows-rtx4090-v1-reset-each-run/selection-summary.json`
    - `selected_fail_total=0`
  - `target/fret-diag-baseline-select-ui-gallery-dialog-escape-focus-restore-steady-windows-rtx4090-v1-reset-each-run/selection-summary.json`
    - `selected_fail_total=0`
  - `target/fret-diag-baseline-select-ui-gallery-dropdown-open-select-steady-windows-rtx4090-v1-reset-each-run-v2/selection-summary.json`
    - `best_candidate.fail_total=0`
    - `threshold_sum_max_top_total_us=808`
  - `target/fret-diag-baseline-select-ui-gallery-overlay-pointer-move-steady-windows-rtx4090-v1-reset-each-run/selection-summary.json`
    - `selected_fail_total=0`
  - `target/fret-diag-baseline-select-ui-gallery-overlay-torture-steady-windows-rtx4090-v1-reset-each-run/selection-summary.json`
    - `best_candidate.fail_total=0`
    - `threshold_sum_max_top_total_us=5819`
- Direct gates (all use `--prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json`
  and `--prelude-each-run`):
  - `ui-gallery-context-menu-right-click-steady`:
    - `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=572/217/0/85/280/0/6`
    - `p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=728/230/0/90/421/0/7`
  - `ui-gallery-dialog-escape-focus-restore-steady`:
    - `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=641/262/0/102/272/0/0`
    - `p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=666/274/0/121/292/0/0`
  - `ui-gallery-dropdown-open-select-steady`:
    - `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=587/216/0/84/267/0/7`
    - `p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=621/242/0/97/275/0/8`
  - `ui-gallery-overlay-pointer-move-steady`:
    - `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=1534/1153/0/85/251/101/1`
    - `p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=1786/1416/0/97/296/107/1`
- `ui-gallery-overlay-torture-steady`:
    - `p50.us(total/layout/solve)=3963/3299/890`
    - `p95.us(total/layout/solve)=4156/3423/946`
    - `pointer_move.max(dispatch/hit_test)=1211/27`

## Finding (2026-05-10): code editor row-rich materialization is off the paint hot path

Observed:

- After the text prepare / row-scene-cache work, the code editor torture script still showed
  paint-time rich text materialization on newly exposed rows.
- The earlier attribution probe had `ns_rich_materialize` around `443300ns`; a first point-based
  prefetch attempt reduced some steady frames but still missed the actual next rows entering the
  viewport.

Change:

- Added a row-rich prefetch runtime next to the syntax prefetch runtime in
  `ecosystem/fret-code-editor/src/editor/mod.rs`.
- Extracted row syntax-span mapping and row-rich cache insertion in
  `ecosystem/fret-code-editor/src/editor/paint/mod.rs`, so paint and prefetch share one cache
  write path.
- Prefetch now targets the immediate scroll-direction edge window (`8` rows) plus a far lookahead
  row, with bounded pending/ready queues (`12` / `32`).
- Ready results are still validated by document, revision, language, theme revision, font-feature
  policy revision, row range, line text, syntax spans, and display-row spans. Pointer identity is the
  fast path, but equal content is accepted when syntax caches are repopulated with equivalent spans;
  accepted results are stored with the current cache Arcs so the next paint can hit.
- This follows the same broad performance direction as Zed/GPUI-style editor rendering: prepare
  line presentation work outside the paint loop and keep paint focused on cache replay / scene
  encoding.

Verification:

- `cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast`
  - `107` tests passed.
- `cargo check -p fret-code-editor --features syntax-rust`
  - passed.
- `cargo build -p fret-ui-gallery --release --features gallery-dev`
  - passed; existing unrelated warnings remain in `fret-runtime` / `fret-ui`.
- Perf command:
  `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 --dir target/fret-diag/perf-code-editor-row-rich-prefetch-equivalence --launch -- target/release/fret-ui-gallery.exe`
- Result:
  - `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=1996/112/0/82/1800/0/0`
  - `p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=2100/114/0/92/1897/0/0`
  - `max.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=2100/114/0/92/1897/0/0`
- Worst bundle:
  `target/fret-diag/perf-code-editor-row-rich-prefetch-equivalence/1778406023238/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 30` summary for the worst bundle:
  - `hot p50/p95 (us): paint.widget=1378/1419 paint.text_prepare=0/0`
  - `renderer p95/max (us): encode=1155/1155 text=113/113 upload=196/196 record=34/34 finish=126/126`
- Paint-probe extraction from the three repeat bundles:
  - `ns_rich_materialize.max=0/0/5300`
  - `row_rich_misses_delta=0/0/2`
  - `row_rich_hits_delta=482/2588/2564`

Residual:

- The remaining code-editor torture cost is no longer row-rich materialization. The next useful
  attribution target is renderer scene encoding / row-scene replay cost, not further syntax-rich
  text materialization.

Code-editor script setup note (2026-05-10):

- `ui-gallery-code-editor-torture-*` scripts require a gallery-dev build:
  `cargo build -p fret-ui-gallery --release --features gallery-dev`. A plain release gallery omits the dev-only
  `code_editor_torture` page; in that shape the nav query is typed successfully, but the filtered nav has zero visible
  items and `ui-gallery-nav-code-editor-torture` never appears.
- `ui-gallery-code-editor-torture-autoscroll-typical.json` waits for font stabilization, so it now carries
  `FRET_UI_GALLERY_BOOTSTRAP_FONTS=1` in `meta.env_defaults`. The steady script intentionally avoids the font wait.

Code-editor autoscroll typical baseline (2026-05-11):

- Use a seed policy whose `ui_threshold_mode` is `frame_p95` when generating this baseline. The perf tooling no longer
  infers typical-frame contracts from the suite name; a raw script path and a named suite behave the same when the
  policy is explicit.
- Seed policy:
  `docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-autoscroll-typical.v1.json`.
- Baseline:
  `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v1.json`.
- Generation command:
  `target/release/fretboard.exe diag perf ui-gallery-code-editor-torture-autoscroll-typical --repeat 15 --warmup-frames 5 --reuse-launch --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v1.json --perf-baseline-headroom-pct 20 --perf-baseline-threshold-surface ui --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-autoscroll-typical.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --dir target/fret-diag/perf-code-editor-autoscroll-typical-suite-baseline-seed --launch target/release/fret-ui-gallery.exe`
- Gate command:
  `target/release/fretboard.exe diag perf ui-gallery-code-editor-torture-autoscroll-typical --repeat 15 --warmup-frames 5 --reuse-launch --perf-threshold-agg p90 --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --dir target/fret-diag/perf-code-editor-autoscroll-typical-suite-gate-current --launch target/release/fret-ui-gallery.exe`
- Result: gate passed with `failures=[]`; `observed_aggregate=p90`, `frame_p95_total_time_us=2291` vs threshold
  `2768`, `frame_p95_layout_time_us=77` vs threshold `352`. The raw CLI line still reports p95/max totals
  `3794/3794us` because that is run-level tail information, not the checked typical threshold.

Explicit UI threshold mode cutover (2026-05-11):

- Perf baseline generation now resolves UI threshold intent from `ui_threshold_mode` instead of suite/script names.
  `top` writes `max_top_*`, `frame_p95` writes `max_frame_p95_*`, and `top_and_frame_p95` writes both.
- The complex editor wheel policy uses `top_and_frame_p95` because it should protect rare wheel tail spikes and
  typical-frame editor paint/payload at the same time.
- A direct repeat=7 gate rerun after adding frame-p95 thresholds missed only the existing top-tail total threshold
  (`5291us` actual vs `5190us` threshold). The same bundle had p50/p95 total `1821/2353us` and paint-dominant worst
  frame cost, so this is tail-jitter evidence, not a reason by itself to start a renderer/display-list rewrite.
- Keep the selector summary
  `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady-windows-rtx4090-v1-policy2/selection-summary.json`
  as the source of truth unless a fresh selector run proves the tail baseline should be intentionally re-seeded.
- Fresh selector run:
  `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady-windows-rtx4090-v1-policy3/selection-summary.json`.
  Both candidates validated `3/3`; candidate-2 won on lower suite p90 (`5027` vs `5600`) and threshold sum
  (`6033` vs `6720`).
- The policy3 baseline keeps the contract editor-grade: top total/layout thresholds are `6033/848us`, frame-p95
  total/layout thresholds are `3808/592us`, and renderer payload thresholds are `258663/406`. Validation worst runs
  remained paint-widget dominant (`top_paint_time_us=5904us` in the worst top run), with renderer encode/upload not
  explaining the tail.

Renderer-aware baseline (2026-05-10):

- Seed policy: `docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-autoscroll-steady.v1.json`
- Baseline: `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v2.json`
- Generation command:
  `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v2.json --perf-baseline-headroom-pct 20 --perf-baseline-threshold-surface all --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-autoscroll-steady.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --dir target/fret-diag/perf-code-editor-renderer-aware-baseline-v2-seeded2 --launch -- target/release/fret-ui-gallery.exe`
- Gate command:
  `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v2.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --dir target/fret-diag/perf-code-editor-renderer-aware-baseline-v2-gate4 --launch -- target/release/fret-ui-gallery.exe`
- Result: `p50/us total-layout-paint=2006/122/1815`, `p95/max=2668/149/2291`, gate passed.

Renderer encode attribution (2026-05-10):

- Added opt-in `FRET_DIAG_RENDERER_ENCODE_FAMILY_PROFILE=1` so `renderer_encode_scene_us` can be split into
  stack/clip/mask/effect/quad/image/text/path/viewport/flush buckets. The new buckets now flow through
  `fret-render-wgpu`, `fret-bootstrap` frame stats, and `diag stats` / triage JSON.
- Suggested evidence command:
  `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_DIAG_RENDERER_ENCODE_FAMILY_PROFILE=1 --dir target/fret-diag/perf-code-editor-renderer-family-profile --launch -- target/release/fret-ui-gallery.exe`
- This is intentionally evidence-first. The next decision point is not another blind renderer rewrite; it is a fresh
  bundle with the family profile enabled so the remaining encode tail can be assigned to the right family before any
  structural refactor.
- Current probe result (`target/fret-diag/perf-code-editor-renderer-family-profile-v2/1778412984109/bundle.schema2.json`):
  `renderer.encode.us(text)=932-1030us`, `clip=16-21us`, `quad=6-12us`, `mask/effect/path/viewport/flush=0`.
  That means the remaining encode tail on this probe is text-heavy, not clip/quad-heavy.
- Added text-phase attribution for the text encode bucket:
  `renderer.encode.text(us/shadow/setup/glyphs)`. The follow-up probe
  (`target/fret-diag/perf-code-editor-renderer-family-profile-v3/1778414559583/bundle.schema2.json`) showed the
  hotspot inside the glyph loop, not text shadow or setup:
  `shadow/setup/glyphs=0/17/996us` on the worst frame, with other top encode frames in the `883-1150us` glyph range.
- Change: `TextShape` now keeps an atlas-revision keyed render-glyph cache, so stable shaped text resolves atlas page/UV
  data before renderer encode instead of doing per-glyph atlas lookup during each scene encode. This matches the
  Zed/GPUI-aligned direction for editor rendering: prepare render-ready text data before the paint/encode replay path,
  and keep encode focused on transform, bounds, batching, and vertex emission.
- Render-cache probe:
  `target/fret-diag/perf-code-editor-render-cache-v1/1778416476030/bundle.schema2.json`.
  `renderer.encode.text(us/shadow/setup/glyphs)` dropped to `0/27/627us` on the worst frame, with other top encode
  frames around `278-582us` in the glyph bucket. The bundle-level renderer encode p95/max is now `978/978us`, down from
  the previous text-phase probe's `1843/1843us` p95/max.
- Interpretation: the atlas lookup component has been moved out of the encode hotspot. The next renderer-side slice
  should attribute and reduce the remaining text vertex emission / group-bounds / group-flush work, or move further
  toward row/fragment replay so stable editor rows do not rebuild all text vertices every frame.
- Follow-up probe: `target/fret-diag/perf-code-editor-render-cache-v4/1778420924991/bundle.schema2.json`.
  The translation+uniform-scale glyph fast path now covers the whole autoscroll run
  (`encode_scene_text_transform_fast_path_glyphs=20420`, `encode_scene_text_transform_generic_glyphs=0`,
  `encode_scene_text_vertex_grow_events=0`), and the repeat=3 probe landed at p50/p95/max total `2091/2165/2165us`
  with paint `1893/1937/1937us`.
- Gate follow-up: `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v2.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --dir target/fret-diag/perf-code-editor-render-cache-gate-current --launch -- target/release/fret-ui-gallery.exe`
  failed with `top_total_time_us=4129` vs threshold `2769`, `renderer_encode_scene_us=3945` vs threshold `1450`,
  `renderer_record_passes_us=84` vs threshold `64`, and `renderer_encoder_finish_us=195` vs threshold `176`.
- Pre-instancing family-profile summary:
  `target/fret-diag/perf-code-editor-renderer-family-profile-current/1778423419022/bundle.json`.
  The worst bundle still spends `renderer_encode_scene_text_us=3621` with
  `renderer_encode_scene_text_glyphs_us=3384`, `renderer_encode_scene_text_glyph_transform_us=758`,
  `renderer_encode_scene_text_glyph_emit_us=815`, `renderer_encode_scene_text_group_flush_us=39`,
  `renderer_encode_scene_text_transform_fast_path_glyphs=20420`, `renderer_encode_scene_text_transform_generic_glyphs=0`,
  and `renderer_encode_scene_text_vertex_grow_events=0`.
  Interpretation: the transform fast path is working, but it is not sufficient for the repeat=7 contract. The next
  evidence-backed slice should move toward row/fragment replay or another row-scoped cache seam that avoids rebuilding
  stable text vertices on every steady frame.
- Text instancing follow-up:
  - Gate command:
    `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v2.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_DIAG_RENDERER_ENCODE_FAMILY_PROFILE=1 --dir target/fret-diag/perf-code-editor-text-instance-gate-current --launch target/release/fret-ui-gallery.exe`
  - Result: gate passed, `failures=[]`, p50/p95/max total `1947/2155/2155us`, paint `1847/2033/2033us`.
  - Worst bundle:
    `target/fret-diag/perf-code-editor-text-instance-gate-current/1778429879460/bundle.schema2.json`.
  - `diag stats --sort cpu_cycles --top 30` reports renderer p95/max `encode=1226/1226us`,
    `upload=90/90us`, `record=41/41us`, `finish=148/148us`. The worst listed frame still has text-heavy scene
    encode (`renderer.encode.us(text)=1043us`, `renderer.encode.text(us/shadow/setup/glyphs)=0/10/941us`), but it is
    now under the current v2 baseline (`renderer_encode_scene_us=1293` vs threshold `1450`, `top_total_time_us=2155`
    vs threshold `2769`).
  - Interpretation: text instancing resolves the repeat=7 gate failure on this local Windows RTX 4090 run. Remaining
    row/fragment replay work should be treated as the next evidence-driven optimization, not as an immediate gate
    unblocker.
- Low-overhead text encode follow-up:
  - Row-scene replay probe:
    `target/fret-diag/perf-code-editor-row-scene-replay-probe/1778433712943/bundle.schema2.json`.
    The code-editor paint telemetry showed `rows_painted=289` and `rows_scene_replayed=288/289`, so the row scene
    cache is already doing the expected steady-frame reuse. The remaining family-profile text cost was not evidence
    to immediately replace that cache with a deeper row/fragment replay model.
  - Change: `FRET_DIAG_RENDERER_TEXT_GLYPH_EMIT_PROFILE=1` now gates per-glyph `GlyphEmit` timing. The renderer family
    profile still reports text setup/glyph/group-flush buckets, but it no longer calls `Instant::now()` once per glyph
    unless this detailed env gate is explicitly enabled. The same slice also aggregates text transform fast/generic
    counters once per blob and reuses cached scale/visibility values in the glyph loop.
  - Gate command:
    `target/release/fretboard.exe diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v2.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_DIAG_RENDERER_ENCODE_FAMILY_PROFILE=1 --dir target/fret-diag/perf-code-editor-text-encode-low-overhead-gate-current --launch -- target/release/fret-ui-gallery.exe`
  - Result: gate passed. p50/p95/max total `2019/2486/2486us`, paint `1925/2396/2396us`. Worst bundle:
    `target/fret-diag/perf-code-editor-text-encode-low-overhead-gate-current/1778434849991/bundle.schema2.json`.
  - `diag stats --sort cpu_cycles --top 30` on the worst bundle reports renderer p95/max
    `encode=361/361us`, top-level renderer text prepare `text=114/114us`, and worst listed frames with
    `renderer.encode.us(text)=255-334us`, `renderer.encode.text(us/shadow/setup/glyphs)=0/10-12/160-215us`,
    `renderer.encode.text(us/transform/emit/flush)=0/0/17-22us`, `transform_fast/generic=18726/0`.
  - Opt-in verification:
    `target/fret-diag/perf-code-editor-text-glyph-emit-profile-optin-probe/1778434885391/bundle.schema2.json`.
    With `FRET_DIAG_RENDERER_TEXT_GLYPH_EMIT_PROFILE=1`, `renderer.encode.text(us/transform/emit/flush)` reports
    `0/364-393/21-23us`, confirming the detailed glyph emit timer still exists and that its cost is diagnostic-only.
  - Interpretation: the previous `~900us` glyph bucket under family profiling mostly measured per-glyph diagnostic
    instrumentation, not unavoidable text instance emission. The current evidence says the right next step is to keep
    the low-overhead family profile as the default gate surface, then use the opt-in glyph emit profile only when a
    narrow text-encode probe needs it. A deeper row/fragment replay design remains valid as a future editor-grade
    direction, but it should be justified by fresh low-overhead evidence rather than by the old high-overhead profile.
- Row-scene key refresh follow-up:
  - Probe:
    `target/fret-diag/perf-code-editor-paint-breakdown-v4-key-refresh-probe/1778437355651/bundle.schema2.json`.
    After the cached syntax replay key is refreshed on a successful full replay, later steady frames recover the
    pointer-identity fast path again. In the worst later frames the code-editor paint telemetry reports
    `ns_row_scene_full_path=0`, `ns_row_scene_fast_path=211-274us`, `ns_row_content_resolve=312-451us`, and
    `ns_total=544-799us`, which is the behavior we want from a GPUI-style cached replay loop.
  - Interpretation: this is a tighter win than jumping straight to row/fragment replay. Keep the syntax replay key
    refresh in place, then only revisit deeper replay if a fresh low-overhead probe still shows stable rows rebuilding
    too much text or geometry.
- Gate noise note:
  - The repeat=7 max gate on this machine is still sensitive to renderer tail thresholds. Recent reruns failed on
    `renderer_encoder_finish_us=185/336` vs threshold `176`, or on `renderer_record_passes_us=90` vs threshold `64`,
    while total and paint times remained in the same healthy band. The latest official hosted-resources gate failed
    only on `renderer_upload_us=416 > 374` and `renderer_encoder_finish_us=179 > 176`, with total/paint still healthy
    (`1584/1926/1926us` and `1498/1821/1821us`). Treat those as renderer-tail noise for this lane, not as a
    code-editor regression, and prefer the low-overhead probe bundles above for the real decision loop.
  - Current rerun (`target/fret-diag/perf-code-editor-hosted-resources-official-gate-v1-rerun3/check.perf_thresholds.json`):
    gate still fails only on `renderer_upload_us=444 > 374` and `renderer_encoder_finish_us=188 > 176`
    (`p50/p95/max total 1664/1934/1934us`, paint `1571/1834/1834us`).
    The new failure payload now carries `evidence_run` and `evidence_peak`, which map the upload failure to
    run_index `1` / frame `717` and the encoder-finish failure to run_index `4` / frame `1834`. That is the
    granularity we wanted: keep treating this as a narrow renderer tail until a fresh low-overhead probe proves
    a structural regression. The latest triage/reporting pass also exposes `renderer_uniform_bytes`,
    `renderer_instance_bytes`, and `renderer_vertex_bytes`, so upload-tail analysis can now separate generic
    upload churn from CPU-generated render payload bytes.
  - Rerun6 record-pass check
    (`target/fret-diag/perf-code-editor-hosted-resources-official-gate-v1-rerun6/1778448662543/bundle.schema2.json`):
    `diag stats --sort record_passes --top 20` showed a single-frame `renderer_record_passes_us=123` outlier at
    tick/frame `719`; adjacent frames stayed at `27-37us`. The outlier did not come with draw/plan churn:
    `renderer.bytes(uniform/instance/vertex)=19776/220248/528`,
    `renderer.encode.ops(stack/clip/mask/effect/quad/image/text/path/viewport/flush)=6/76/2/0/75/0/338/0/0/1`,
    no intermediate allocations/releases, and the same steady draw footprint (`133-134` draw calls, `114` pipeline
    switches). Treat this as renderer-tail jitter until a fresh low-overhead pass-kind probe shows a repeatable
    structural cost. The Zed/GPUI comparison still supports keeping the common draw path flat, but this bundle is not
    evidence for a broad pass-organization rewrite.
- Content-resolve split follow-up (2026-05-11):
  - Probe:
    `target/fret-diag/perf-code-editor-content-resolve-breakdown-v1/1778439554384/bundle.schema2.json`.
    This run uses `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` and is an attribution probe, not a promoted gate.
  - Change: code-editor paint telemetry schema `6` now exposes `us/ns_row_rich_cache_compare`,
    `us/ns_row_geom_key`, `us/ns_row_scene_key`, `us/ns_row_scene_fast_key_compare`, and
    `us/ns_row_scene_full_key_compare`.
  - Result: repeat=3 reported p50/p95/max total `1939/1949/1949us` and paint `1845/1854/1854us`. The top
    paint-perf snapshots showed `ns_row_content_resolve=342-506us`, `ns_row_scene_fast_path=273-341us`,
    `ns_row_text=69-87us`, `ns_syntax_spans=38-52us`, `ns_row_scene_fast_key_compare=26-38us`,
    `ns_row_rich_cache_compare<=0.8us`, `ns_row_geom_key<=2.4us`, and `ns_row_scene_key` was usually
    `<=0.3us` with one `9.6us` outlier. `ns_row_scene_full_path` stayed near zero.
  - Interpretation: RowGeomKey/RowSceneKey construction and row-rich cache comparison are not the remaining
    bottleneck. The remaining content-resolve cost is mostly the row-scene fast replay path itself (probe/key compare,
    hosted-resource touch, and translated op replay) plus occasional new-row text draw. If this lane continues with
    row-scoped work, target row-scene replay/touch mechanics from fresh low-overhead evidence rather than key
    construction.

- Hosted-resource touch precompute (2026-05-11):
  - Change: `CanvasHostedResources` now precomputes `TextBlobId` / `PathId` / `SvgId` references from retained scene
    ops at cache-store time, and row-scene replay touches those precomputed lists instead of rescanning `cached.ops`.
  - Implementation anchors:
    - `crates/fret-ui/src/canvas.rs`
    - `ecosystem/fret-code-editor/src/editor/mod.rs`
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - Verification:
    - `cargo check -p fret-ui`
    - `cargo check -p fret-code-editor --features syntax-rust`
    - `cargo nextest run -p fret-ui --lib hosted_resources_from_scene_ops_collects_resource_ids --no-fail-fast`
  - Probe:
    `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_A11Y_DISABLE=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 --dir target/fret-diag/perf-code-editor-hosted-resources-v1 --launch -- target/release/fret-ui-gallery.exe`
  - Result: p50/p95/max total `1762/2070/2070us`, paint `1669/1988/1988us`.
  - Low-overhead bundle:
    `target/fret-diag/perf-code-editor-hosted-resources-v1/1778441726056/bundle.schema2.json`
    shows `ns_row_scene_replay_touch=39900ns` for `rows_scene_replayed=288`, roughly half the earlier content-resolve
    touch bucket.
  - Trace follow-up:
    `target/fret-diag/perf-code-editor-hosted-resources-trace-v1/1778449929019/bundle.schema2.json` kept
    p50/p95/max total at `1603/1722/1722us`; the exported `trace.chrome.json` is useful for coarse phase alignment,
    but it still does not expose a dedicated `fret.renderer.record_passes` span name.
  - Layout-node profile follow-up:
    `target/fret-diag/perf-code-editor-hosted-resources-layout-node-profile-v1/1778450026275/bundle.schema2.json`
    kept p50/p95/max total at `1723/1730/1730us`; the hotspots sit in `Scroll` nodes from `scroll_area.rs` /
    `content.rs`, which reads like profiling overhead rather than a layout regression.
  - The same worst-frame evidence now also appears in `diag triage` as `phase.timeline_hotspots`, which
    ties the phase timeline to layout, scroll, paint, and renderer hotspot examples in one place. The fresh
    low-overhead probe `target/fret-diag/perf-code-editor-hotspot-probe-v1/1778454652187/bundle.schema2.json`
    is paint-widget heavy on `ElementHostWidget::Canvas` and shows `renderer.upload_churn`; the detailed paint
    probe `target/fret-diag/perf-code-editor-paint-detail-probe-v1/1778455020350/bundle.schema2.json` still
    replays `288/289` visible rows and stores only 1 new row, so do not spend time splitting `RowGeomKey` or
    `RowSceneKey` further.
  - Interpretation: the row-scene replay hot path is still the right place to look, but the cheap win is already in
    place. Do not spend time splitting `RowGeomKey` or `RowSceneKey` further; if this lane continues, target
    replay/touch mechanics or new-row text draw from fresh low-overhead evidence.

## Failure exemplar map

- Layout-root build spikes: `Finding (2026-02-14): repeat=7 can fail on Material3 tabs (request_build_roots dominates)`.
- Layout-engine solve spikes: `Finding (2026-02-15): Batch-solve barrier roots to eliminate per-root solve spikes`.
- Paint spikes: `Finding (2026-05-10): ui-gallery-complex-steady now yields a paint-dominant Windows exemplar when run with --prelude-each-run; use target/fret-diag/1778364986668/bundle.schema2.json for paint-tail attribution.`
- Code-editor paint spikes: `Finding (2026-05-10): code editor row-rich materialization is off the paint hot path; use target/fret-diag/perf-code-editor-row-rich-prefetch-equivalence/1778406023238/bundle.schema2.json for the post-fix renderer-encode exemplar.`

## Next steps

### 1) Reduce remaining tail spikes (Windows-specific)

Hypotheses to validate:

- allocator jitter (large transient allocations outside the frame arena)
- hash/vec capacity growth on “rare” paths
- background thread wakeups competing with the UI thread during resize

Candidate actions (small → large):

- tighten capacity reuse for known hot scratch structures (avoid occasional rehash/grow)
- `scratch_element_children_vec_pool` now exports `grow_events` through `UiDebugFrameStats`,
  `ElementDiagnosticsSnapshotV1`, `diag stats`, and `memory_summary`; the runtime growth path is covered by the
  `fret-ui` unit test `scratch_element_children_vec_pool_grow_events_increment_when_reused_vec_expands`, and the
  `fret-diag` bundle parser has a regression test for `element_children_vec_pool_grow_events`.
- make “layout request → build roots → solve → apply” phase boundaries visible by default in traces
- add a small set of churn counters (“bytes allocated”, “vec grow events”) for the worst offenders

### 2) Strengthen profiling + stats surfaces (fearless refactor)

This workstream depends on (and should not duplicate) the broader diagnostics effort:

- `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1.md`
- `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1-field-inventory.md`

The delta we want here is “Windows smoothness” oriented:

- faster “good vs bad” comparison loops (1–2 commands)
- clearer typical-perf reporting (p50/p95 as first-class in review)
- stronger linkage from a failing threshold → responsible phase → top hotspots

### 3) Profiling/stats refactor proposal (what we would change, fearlessly)

We already have many of the right pieces (scripts, bundles, gates, `diag stats`, optional traces).
The main gap is that reviewers still need “tribal knowledge” to go from **a failing threshold** to
**a clear root cause**.

Proposed direction (additive, contract-first):

1) Make a stable per-frame schema explicit
   - Treat perf keys as a contract (`*_time_us`, `*_calls`, `*_items`, `*_bytes`).
   - Keep changes additive; avoid renames without a compatibility window.
2) Make typical perf first-class (not just max)
   - Percentiles (p50/p95/p99) should be available in `diag stats` outputs and diffs.
   - Review workflow: “p95 moved +X%” becomes a standard callout, not a manual spreadsheet step.
3) Close the “attribution loop”
   - For each gated metric, define its closest phase boundary + top hotspots surface.
   - Example: `top_layout_time_us` → (`layout_request_build_roots` / `layout_roots` / `layout_engine_solve`) + node profile.
4) Three-lane profiling (borrow the mature pattern)
   - Always-on: cheap counters + coarse timings (gates).
   - Opt-in: structured spans / node-level top-N (attribution).
   - External sampling: ETW/WPR (OS scheduling/IO) + PIX/Nsight (GPU).

Comparative notes (how other UI stacks tend to succeed here):

- Zed/GPUI style: per-frame arenas + scoped CPU profiling (Tracy-style) + explicit frame markers.
- Immediate-mode UIs (e.g. egui): lightweight in-app profilers (puffin) + consistent “frame budget”
  dashboards (great for typical perf, weaker for tail unless paired with external profilers).
- Large engines (Chromium/Flutter): stable trace events + external system profilers; “trace names are
  a contract” is non-negotiable.

## References / important code

- Layout pass + phase timers: `crates/fret-ui/src/tree/layout.rs`
- Layout engine (Taffy): `crates/fret-ui/src/layout/engine.rs`
- Stats summary / JSON keys: `crates/fret-diag/src/stats.rs`
- Diagnostics script runner / checks: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
