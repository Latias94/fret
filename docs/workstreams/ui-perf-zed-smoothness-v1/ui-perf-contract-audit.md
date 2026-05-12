# UI Performance Contract Audit

Status: Active audit; goal not complete.
Date: 2026-05-11

## Objective

Establish and maintain an editor-grade performance contract comparable to Zed/GPUI and egui:

- representative scripts are mapped to checked-in baselines and gates,
- baseline evidence tracks `p50`, `p95`, and `max`,
- hot-path churn is reduced only when measured evidence points at a real bottleneck,
- fearless refactors remain reversible through commands, logs, and gates.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Representative editor-grade scripts are listed. | `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md` maps steady gallery, resize, code editor resize, code editor autoscroll, complex code editor wheel, view-cache resize torture, pointer move/hit-test, and renderer/effects churn. | Covered: the matrix now classifies the view-cache resize torture scripts under `ui-resize-probes` and adds the complex editor wheel stressor as a dedicated contract surface. |
| Baseline rows can record `p50/p95/max`. | `crates/fret-diag/src/diag_perf/baseline_rows.rs` writes `measured_p50`; smoke output `target/fret-diag/codex-p50-baseline-smoke/baseline.json` included `measured_p50`. | Tooling covered for new baselines. |
| Renderer payload metrics can become hard contract fields. | `crates/fret-diag/src/diag_perf/stats_rows.rs`, `runs_rows.rs`, `reporting.rs`, `baseline_rows.rs`, `thresholds.rs`, `crates/fret-diag/src/compare.rs`, and `crates/fret-diag/src/diag_perf_baseline.rs` now propagate `renderer_instance_bytes` and `renderer_encode_scene_text_ops` through perf JSON, baseline JSON, baseline parsing, threshold rows, and threshold failures. The code-editor autoscroll v4 and complex wheel v1 baselines are checked-in `ui-renderer-payload` contracts. | Covered for the editor paint contracts; older baselines that predate these fields remain valid time-only contracts until intentionally re-seeded. |
| UI threshold mode is explicit. | `crates/fret-diag/src/perf_seed_policy.rs`, `crates/fret-diag/src/diag_perf.rs`, `crates/fret-diag/src/diag_perf/baseline_rows.rs`, and `tools/perf/diag_perf_baseline_select.py` now carry `ui_threshold_mode` (`top`, `frame_p95`, or `top_and_frame_p95`) through policy, CLI overrides, selector summaries, and baseline rows. | Covered for new baselines; old baselines remain valid, but new typical-frame contracts must not rely on suite-name inference. |
| Checked-in baselines actually contain `p50`. | Scan on 2026-05-11 after the Windows resize/code-editor v2 promotions, autoscroll steady v4 promotion, autoscroll typical v2 promotion, complex wheel v1 promotion, view-cache toggle v1 promotion, virtual-list v1 promotion, menubar keyboard nav v1 promotion, Material 3 tabs v1 promotion, and hover-layout v1 promotion: 79 baseline files, 315 max-bearing perf rows, 19 rows with `measured_p50`. | Partially covered. `ui-resize-probes.windows-rtx4090.v2.json`, `ui-code-editor-resize-probes.windows-rtx4090.v2.json`, `ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v4.json`, `ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json`, `ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json`, `ui-gallery-view-cache-toggle-perf-steady.windows-rtx4090.v1.json`, `ui-gallery-virtual-list-torture-steady.windows-rtx4090.v1.json`, `ui-gallery-menubar-keyboard-nav-steady.windows-rtx4090.v1.json`, `ui-gallery-material3-tabs-switch-perf-steady.windows-rtx4090.v1.json`, and `ui-gallery-hover-layout-torture-steady.windows-rtx4090.v1.json` are covered; other checked-in baselines need intentional re-seeding if p50 must be checked in. |
| Gates use the correct machine baseline. | `tools/perf/diag_resize_probes_gate.py` and `.sh` choose Windows RTX 4090 or macOS baseline by host platform. | Covered for resize helpers; other gate helpers remain explicit-baseline by design. |
| Gates use normalization hooks. | Resize gate helpers and baseline selectors now apply `tooling-suite-prewarm-fonts.json` and `tooling-suite-prelude-reset-diagnostics.json` by default. | Covered for the updated helpers/selectors. |
| A short real gate proves the helper path works. | `target/fret-diag/codex-resize-gate-default-hooks-smoke/summary.json`: `ui-resize-probes`, attempts=1, repeat=1, PASS, Windows baseline selected, default hooks recorded. | Smoke covered, not a full formal gate. |
| Full formal gates are green after the helper changes. | `target/fret-diag/codex-resize-flex-patch-gate-r7-v2-headroom30/summary.json`: Windows `ui-resize-probes` v2 passed attempts=3 repeat=7 with `pass_attempts=3`. `target/fret-diag-code-editor-resize-probes-windows-rtx4090-v2-gate-r7/summary.json`: `ui-code-editor-resize-probes` v2 passed attempts=3 repeat=7 with `pass_attempts=2`. | Covered for both resize gates. |
| Zed/GPUI and egui comparison remains explicit. | `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md` plus the contract matrix reference pressure column. | Covered as a design map; still needs updates when new gaps close. |
| Churn reduction is evidence-led. | Perf log entries show measured view-cache harness virtualization, code editor resize attribution, and decisions not to start broad root-solve quantization or `WindowedRowsSurface` rewrites without evidence. | Covered for recent work. |
| Baseline maintenance rules are documented. | `docs/workstreams/perf-baselines/README.md` defines machine tags, re-seed criteria, required hooks, selector workflow, validation workflow, no-silent-threshold-loosening guard, and review checklist. | Covered. |
| Completion criteria are unambiguous. | This audit maps requirements to evidence and gaps. | Covered, with open gaps below. |

## Current Evidence Snapshot

- Recent commits:
  - `e9b6ebf2d4 perf(code-editor): fix soft-wrap syntax prefetch mapping`
  - `c47df6f34c perf(baselines): promote code editor payload contract`
  - `ece88ce0e6 feat(diag): support renderer payload perf baselines`
  - `5b2038cf7d feat(diag): gate renderer payload perf metrics`
  - `234becda06 docs(perf): record gpui reference pass for editor canvas`
  - `6be5cd33fe docs(perf): refresh editor-grade contract audit`
  - `5592215523 feat(diag): record p50 in perf baselines`
  - `7506e02351 fix(perf): select resize baselines by host platform`
  - `fd45b0d1cf fix(perf): normalize resize gate suite hooks`
  - `5998e1df82 docs(perf): document baseline maintenance policy`
  - `0121e7f10a fix(perf): separate baseline threshold surfaces`
  - `380db5d44d perf(ui): profile flex-wrap layout patch`
  - `a58277f72 feat(diag): surface hotspot and scratch growth signals`
  - `0ebcebd04 docs(perf): record code-editor hotspot evidence`
- Short resize gate smoke:
  - Summary: `target/fret-diag/codex-resize-gate-default-hooks-smoke/summary.json`
  - Result: PASS, `failures=0`
  - `drag-jitter`: `top_total/layout/solve=1728/1103/661us`
  - `resize-stress`: `top_total/layout/solve=4021/1664/671us`
- Promoted Windows `ui-resize-probes` v2 re-seed:
  - 20% headroom remained too tight under repeat=7 validation:
    `target/fret-diag-baseline-select-ui-resize-probes-windows-rtx4090-v2-flexpatch/selection-summary.json` recorded
    `best_candidate.fail_total=3`.
  - 30% headroom selected candidate 2 with `fail_total=0`:
    `target/fret-diag-baseline-select-ui-resize-probes-windows-rtx4090-v2-headroom30-flexpatch/selection-summary.json`.
  - Matching formal gate passed:
    `target/fret-diag/codex-resize-flex-patch-gate-r7-v2-headroom30/summary.json`, attempts=3 repeat=7,
    `pass_attempts=3`.
- Baseline p50 coverage scan:
  - `BASELINE_FILES=79`
  - `TOTAL_ROWS=315`
  - `TOTAL_ROWS_WITH_P50=19`
  - `TOTAL_ROWS_WITH_P90=164`
  - `TOTAL_ROWS_WITH_P95=164`
  - `TOTAL_ROWS_WITH_MAX=315`
- Promoted Windows `ui-code-editor-resize-probes` v2 re-seed:
  - 20% headroom selector chose candidate 1 with `fail_total=0`:
    `target/fret-diag-baseline-select-ui-code-editor-resize-probes-windows-rtx4090-v2/selection-summary.json`.
  - Matching formal gate passed by majority:
    `target/fret-diag-code-editor-resize-probes-windows-rtx4090-v2-gate-r7/summary.json`, attempts=3 repeat=7,
    `pass_attempts=2`.
  - The failed attempt was paint-dominant, not layout-dominant:
    `top_total_time_us=13800` vs threshold `11282`, while layout/solve were below threshold and
    `diag stats --sort cpu_cycles` showed `paint.widget p95=10922us`.
  - New code-editor hotspot evidence:
    `target/fret-diag/perf-code-editor-hotspot-hint-probe-v1/check.perf_hints.json` flags
    `paint.widget_heavy` on `ElementHostWidget::Canvas` plus `renderer.upload_churn`, while
    `target/fret-diag/perf-code-editor-paint-detail-probe-v1/1778455020350/bundle.schema2.json` still replays
    `288/289` visible rows and stores only 1 new row.
- `ui-gallery-steady` re-seed attempts after `--reuse-launch-per-script` and `--prelude-each-run` normalization:
  - `target/fret-diag-baseline-select-ui-gallery-steady-windows-rtx4090-v3b/selection-summary.json`:
    candidate-1 validated with `fail_total=2`, both failures on `ui-gallery-view-cache-toggle-perf-steady`.
  - `target/fret-diag-baseline-select-ui-gallery-steady-windows-rtx4090-v3c/selection-summary.json`:
    candidate-1 still failed across multiple scripts (`hover-layout`, `dropdown`, `overlay`, `view-cache-toggle`,
    `virtual-list`, `window-resize`), showing the suite is still too broad for a stable single Windows baseline.
- Promoted Windows `ui-gallery-view-cache-toggle-perf-steady` v1 baseline:
  - `target/fret-diag-baseline-select-ui-gallery-view-cache-toggle-perf-steady-windows-rtx4090-v1/selection-summary.json`:
    candidate-1 validated with `fail_total=2`, while candidate-2 validated `3/3` with `fail_total=0`.
  - Checked-in baseline:
    `docs/workstreams/perf-baselines/ui-gallery-view-cache-toggle-perf-steady.windows-rtx4090.v1.json`.
  - p50/p95/max total=`2306/2457/2457us`; thresholds total/layout/solve=`2949/2378/80us`.
- Promoted Windows `ui-gallery-virtual-list-torture-steady` v1 baseline:
  - `target/fret-diag-baseline-select-ui-gallery-virtual-list-torture-steady-windows-rtx4090-v1/selection-summary.json`:
    candidate-1 validated with `fail_total=3`, while candidate-2 validated `3/3` with `fail_total=0`.
  - Checked-in baseline:
    `docs/workstreams/perf-baselines/ui-gallery-virtual-list-torture-steady.windows-rtx4090.v1.json`.
  - p50/p95/max total=`7014/7645/7645us`; thresholds total/layout/solve=`9174/7488/2031`.
- Promoted Windows `ui-gallery-menubar-keyboard-nav-steady` v1 baseline:
  - `target/fret-diag-baseline-select-ui-gallery-menubar-keyboard-nav-steady-windows-rtx4090-v1/selection-summary.json`:
    candidate-1 validated `3/3` with `fail_total=0`, while candidate-2 validated `3/3` with `fail_total=1`.
  - Checked-in baseline:
    `docs/workstreams/perf-baselines/ui-gallery-menubar-keyboard-nav-steady.windows-rtx4090.v1.json`.
  - p50/p95/max total=`1666/3385/3385us`; thresholds total/layout/solve=`4062/3516/731us`.
- Promoted Windows `ui-gallery-material3-tabs-switch-perf-steady` v1 baseline:
  - Requires `cargo build -p fret-ui-gallery --release --features gallery-full` before launching
    `target/release/fret-ui-gallery.exe`.
  - `target/fret-diag-baseline-select-ui-gallery-material3-tabs-switch-perf-steady-windows-rtx4090-v1-policy40/selection-summary.json`:
    candidate-1 and candidate-2 both validated `3/3` with `fail_total=0`; candidate-2 won on p90 (`1924` vs
    `2231`).
  - Checked-in baseline:
    `docs/workstreams/perf-baselines/ui-gallery-material3-tabs-switch-perf-steady.windows-rtx4090.v1.json`.
  - Seed policy:
    `docs/workstreams/perf-baselines/policies/ui-gallery-material3-tabs-switch-perf-steady.v1.json`.
  - p50/p95/max total=`1873/1924/1924us`; thresholds total/layout/solve/pointer_move(dispatch/hit-test)=
    `2694/1610/0/1536/32`.
- Promoted Windows `ui-gallery-hover-layout-torture-steady` v1 baseline:
  - Initial no-policy selectors showed the failure mode was narrow threshold noise, not script instability: the 20%
    run failed on `pointer_move_max_dispatch_time_us` / `top_layout_time_us`, and the 40% run still had small
    pointer/layout failures.
  - Seed policy:
    `docs/workstreams/perf-baselines/policies/ui-gallery-hover-layout-torture-steady.v1.json`.
  - `target/fret-diag-baseline-select-ui-gallery-hover-layout-torture-steady-windows-rtx4090-v1-policy/selection-summary.json`:
    candidate-2 validated `3/3` with `fail_total=0`; candidate-1 had one validation failure.
  - Checked-in baseline:
    `docs/workstreams/perf-baselines/ui-gallery-hover-layout-torture-steady.windows-rtx4090.v1.json`.
  - p50/p95/max total=`998/1285/1285us`; thresholds total/layout/solve/pointer_move(dispatch/hit-test)=
    `1542/248/0/448/32`.
  - Semantic hover gate:
    `cargo run -q -p fretboard-dev -- diag stats target/fret-diag-baseline-select-ui-gallery-hover-layout-torture-steady-windows-rtx4090-v1-policy/candidate-2-baseline/1778476920836/bundle.schema2.json --check-hover-layout-max 0`
    passed; `hover.decl_inv(layout/hit/paint)=0/0/0`.
- Renderer payload contract surface:
  - `renderer_instance_bytes` and `renderer_encode_scene_text_ops` now flow through perf JSON rows, repeat summaries,
    baseline rows, `perf-baseline-from-bundles`, baseline parsing, threshold rows, and threshold failures.
  - `ui_threshold_mode` now explicitly selects `top`, `frame_p95`, or `top_and_frame_p95`, removing the old
    suite-name-derived typical contract inference.
  - Validation: `cargo fmt -p fret-diag --check`; `cargo nextest run -p fret-diag`;
    `python tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`.
  - The checked-in `ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v4.json` baseline is now the
    first payload-aware editor paint contract. Selector evidence:
    `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-autoscroll-steady-windows-rtx4090-v4c/selection-summary.json`.
  - Both v4 selector candidates validated `3/3` with `fail_total=0`; selected thresholds are
    `max_top_total_us=3072`, `max_top_layout_us=320`, `max_renderer_instance_bytes=323482`, and
    `max_renderer_encode_scene_text_ops=611`.
  - The checked-in `ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json` baseline adds the
    typical-frame editor paint/payload contract. Selector evidence:
    `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-autoscroll-typical-windows-rtx4090-v2/selection-summary.json`.
  - Both v2 selector candidates validated `3/3` with `fail_total=0`; selected candidate-1 measured
    p50/p95/max top total=`2563/3603/3603us`, top layout=`77/123/123us`. The hard thresholds are
    frame p95 total/layout/solve=`3360/368/0us` plus payload thresholds
    `max_renderer_instance_bytes=262416`, `max_renderer_encode_scene_text_ops=406`.
  - The checked-in
    `ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json`
    baseline adds the high-stress editor wheel tail + typical-frame contract after setup reset. Selector evidence:
    `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady-windows-rtx4090-v1-clamp-no-loosen/selection-summary.json`.
  - Candidate-1 validated `3/3` with `fail_total=0`, `threshold_loosening_count=0`, and
    `threshold_clamp_count=5`; candidate-2 failed validation `3/3`, including one `top_total_time_us=8514us` tail.
    Selected candidate-1 measured p50/p90/max top total=`2257/4617/4617us`, frame-p95 total=`1730/2968/2968us`,
    hard thresholds top(total/layout/solve)=`6033/848/0us`, frame-p95(total/layout/solve)=`3808/592/0us`,
    pointer dispatch/hit-test=`489/14us`, and payload thresholds instance/text_ops=`258663/406`.
  - Follow-up semantic fix `e9b6ebf2d4` maps soft-wrap syntax prefetch through `DisplayMap::display_row_line(...)`
    before chunking. The worst paint-detail sample in
    `target/fret-diag/perf-complex-editor-wheel-tail-syntax-line-prefetch-v1/1778501381582/bundle.json` drops
    `top_total_time_us` from `5681us` to `3580us`, with `syntax_evict_delta=0` and `row_rich_miss_delta=0`; the
    checked-in baseline still passes with max total `3238us` and max paint `3046us`.
  - Fresh CPU attribution on the post-fix worst bundle shows the remaining hot path is Canvas paint-widget work
    (`paint_widget_time_us=1550`, Canvas hotspot `1250us`) with renderer encode/upload still visible
    (`renderer_encode_scene_us=319`, `renderer_upload_us=78`). That means the next slice should compare the row-scene
    fast replay path against Canvas/renderer payload before changing thresholds again.
  - `diag stats` now surfaces the existing editor `paint_perf` counters as first-class JSON/text attribution and
    prefers `ns_*` counters when present, avoiding the per-row microsecond rounding loss in the original `us_*`
    fields.
    Validation on
    `target/fret-diag/perf-complex-editor-wheel-tail-syntax-line-prefetch-v1/1778501381582/bundle.json` reports
    `code_editor_paint_perf.frames=34`; the worst top frame has `rows_scene_replayed=204`,
    `rows_scene_stored=1`, `us_row_content_resolve=544`, `us_row_scene_fast_path=373`,
    `us_row_scene_fast_probe=63`, `us_row_scene_replay_ops=70`, `us_row_scene_replay_touch=78`,
    `us_row_scene_capture_ops=0`, and `us_text_draw=0`. The summary p95 has `us_total=886`,
    `us_row_content_resolve=636`, `us_row_scene_fast_path=347`, `us_row_text=88`, and `us_text_draw=147`.
    Keep the next optimization focused on measured fast-replay/content/Canvas/renderer cost unless a new stressor
    shows row-scene capture/store as the limiter.
  - Follow-up replay semantics fix: `SceneRecording::replay_ops` now maintains `Scene::text_blob_ids()` for replayed
    text ops, and code-editor row-scene replay uses the new precomputed-index replay API. This matches the GPUI/Zed
    direction where replay rebuilds side collections via the normal scene insertion path. Paint-detail repeat=3
    evidence in
    `target/fret-diag/perf-complex-editor-scene-replay-text-index-v1/1778515050738/bundle.schema2.json` reports
    code-editor p95 `us_row_scene_replay_touch=65`, `us_row_scene_replay_ops=77`, and
    `us_row_scene_fast_path=451`; the newly correct renderer text prepare cost is now visible as p95/max
    `1287/1302us` with text atlas upload/eviction still `0`. The non-instrumented repeat=3 baseline check in
    `target/fret-diag/perf-complex-editor-scene-replay-text-index-baseline-check-v1/1778515146987/bundle.json` passed
    the checked-in v1 contract with worst top total `2859us` and payload `254/192368`.
    Next attribution should inspect renderer text prepare / glyph pinning and possible text-index compaction; do not
    treat the larger text-prepare number as a regression caused by row-scene replay.
  - Follow-up renderer text prepare fix: `TextShape` now stores a pre-deduplicated `GlyphPinKeys` set and renderer
    atlas pinning merges those sets instead of scanning every glyph instance each frame. Paint-detail repeat=3 evidence
    in `target/fret-diag/perf-complex-editor-shape-pin-keys-v1/1778516581210/bundle.schema2.json` improves renderer
    text p95/max from `1287/1302us` to `660/722us`, while top total p50/p95/max becomes `1925/2125/2125us`. The
    non-instrumented repeat=3 baseline check in
    `target/fret-diag/perf-complex-editor-shape-pin-keys-baseline-check-v1/1778516630518/bundle.json` passed the
    checked-in v1 contract with worst top total `2206us`, `top_renderer_prepare_text_us` p50/p95/max `424/426/426us`,
    and payload `254/192368`.
- IMUI hello smoke correctness recheck:
  - `FRET_DIAG=1 FRET_DIAG_DIR=target/fret-diag/imui-hello-demo-screenshot-recheck FRET_DIAG_GPU_SCREENSHOTS=1 cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/local-debug/imui-hello-demo-screenshot.json --dir target/fret-diag/imui-hello-demo-screenshot-recheck --session-auto --timeout-ms 180000 --launch -- cargo run -p fret-demo --bin imui_hello_demo`
  - Screenshot evidence:
    `target/fret-diag/imui-hello-demo-screenshot-recheck/sessions/1778605632348-99852/screenshots/1778606081730-imui-hello-demo/window-4294967297-tick-41-frame-40.png`
  - The recheck now shows visible text again (`Count: 0`, `Increment`, `Enabled: false`, `Enabled`), so the earlier blank Windows smoke is confirmed as pre-fix evidence rather than a WSL-specific symptom.
- Complex editor wheel frame-overlay cache:
  - Before bundle:
    `target/fret-diag/perf-complex-editor-wheel-paint-detail-v1/1778490773008/bundle.schema2.json`.
  - After bundle:
    `target/fret-diag/perf-complex-editor-wheel-overlay-cache-v3-final/1778495502010/bundle.schema2.json`.
  - Paint-detail `ns_total` p50/p95/max improved from `1041.1/1345.3/1371.3us` to `488.8/730.8/832.4us`.
  - `ns_row_overlay` p50/p95/max improved from `523.1/556.0/763.8us` to `6.9/8.2/9.6us`, with
    `ns_frame_overlay_prepare` p50/p95/max=`7.9/9.2/16.7us`.
  - An initial post-optimization re-seed attempt was not promoted:
    `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady-windows-rtx4090-v1-policy4-overlay-cache/selection-summary.json`
    selected candidate-2 with `selected_fail_total=1`; the miss was one `top_total_time_us=4389us` paint-tail
    sample against a `3365us` top threshold while frame p95 total was `2176us`. Keep the checked-in v1 baseline until
    a policy change is intentional.
- Baseline selector threshold-loosening guard:
  - A later post-optimization selector attempt selected a candidate that validated `3/3`, but would have widened the
    complex editor wheel `max_top_total_us` threshold from `6033us` to `6912us`; it was not promoted.
  - `tools/perf/diag_perf_baseline_select.py` now compares candidates against an existing `--baseline-out` file and
    rejects threshold increases/removals unless `--allow-threshold-loosening` is explicitly passed. The selector also
    supports `--clamp-threshold-loosening`, which validates candidates with existing stricter thresholds preserved when
    the candidate's measured value still fits the old contract.
  - Validation: `python -m unittest discover -s tools/perf -p 'test_*.py'`;
    `python tools/perf/diag_perf_baseline_select.py --help`; `git diff --check`.

## Open Gaps

1. The broad `ui-gallery-steady` suite remains evidence-only until it is redefined as a suite-of-contracts or its
   membership is intentionally narrowed. Its former broad-only members are now covered by dedicated Windows contracts;
   do not try to re-promote the broad suite by loosening thresholds.
2. The autoscroll typical v2 and complex wheel v1 contracts cover stricter editor paint/payload surfaces, and the
   complex wheel overlay hotspot now has a narrower frame-derived-state fix. Keep the `WindowedRowsSurface`
   display-list rewrite gated on a future near-threshold or failing stressor where row op replay/capture is the
   measured limiter, not on these passing baselines alone.
3. Keep Linux and any other non-Windows/macOS machine profiles explicit until a checked-in baseline and owner profile exist.

## Audit Conclusion

The goal is not complete. The Windows `ui-resize-probes` and `ui-code-editor-resize-probes` contracts now have
checked-in `measured_p50` evidence and green formal repeat=7 gates, the code-editor autoscroll steady, autoscroll
typical, and complex wheel contracts now have payload-aware baselines with explicit UI threshold modes, and
`ui-gallery-view-cache-toggle-perf-steady`, `ui-gallery-virtual-list-torture-steady`, `ui-gallery-menubar-keyboard-nav-steady`,
`ui-gallery-material3-tabs-switch-perf-steady`, and `ui-gallery-hover-layout-torture-steady` are now dedicated Windows
v1 contracts. The next work should only start a `WindowedRowsSurface` display-list rewrite from a near-threshold or
failing editor paint stressor, and keep non-Windows machine profiles explicit rather than inferring them from the
Windows RTX 4090 contract set.
