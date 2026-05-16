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
| Renderer payload metrics can become hard contract fields. | `crates/fret-diag/src/diag_perf/stats_rows.rs`, `runs_rows.rs`, `reporting.rs`, `baseline_rows.rs`, `thresholds.rs`, `crates/fret-diag/src/compare.rs`, and `crates/fret-diag/src/diag_perf_baseline.rs` now propagate `renderer_instance_bytes` and `renderer_encode_scene_text_ops` through perf JSON, baseline JSON, baseline parsing, threshold rows, and threshold failures. `tools/perf/audit_perf_baselines.py` now also rejects `ui-renderer-payload`, `renderer-payload`, `renderer`, and `all` baselines that omit payload values from `measured_*`, `threshold_seed`, or hard thresholds. The code-editor autoscroll v4 and complex wheel v1 baselines are checked-in `ui-renderer-payload` contracts. | Covered for the editor paint contracts; older baselines that predate these fields remain valid time-only contracts until intentionally re-seeded. |
| UI threshold mode is explicit. | `crates/fret-diag/src/perf_seed_policy.rs`, `crates/fret-diag/src/diag_perf.rs`, `crates/fret-diag/src/diag_perf/baseline_rows.rs`, and `tools/perf/diag_perf_baseline_select.py` now carry `ui_threshold_mode` (`top`, `frame_p95`, or `top_and_frame_p95`) through policy, CLI overrides, selector summaries, and baseline rows. | Covered for new baselines; old baselines remain valid, but new typical-frame contracts must not rely on suite-name inference. |
| Checked-in baselines actually contain `p50`. | Scan on 2026-05-11 after the Windows resize/code-editor v2 promotions, autoscroll steady v4 promotion, autoscroll typical v2 promotion, complex wheel v1 promotion, view-cache toggle v1 promotion, virtual-list v1 promotion, menubar keyboard nav v1 promotion, Material 3 tabs v1 promotion, and hover-layout v1 promotion: 79 baseline files, 315 max-bearing perf rows, 19 rows with `measured_p50`. | Partially covered. `ui-resize-probes.windows-rtx4090.v2.json`, `ui-code-editor-resize-probes.windows-rtx4090.v2.json`, `ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v4.json`, `ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json`, `ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json`, `ui-gallery-view-cache-toggle-perf-steady.windows-rtx4090.v1.json`, `ui-gallery-virtual-list-torture-steady.windows-rtx4090.v1.json`, `ui-gallery-menubar-keyboard-nav-steady.windows-rtx4090.v1.json`, `ui-gallery-material3-tabs-switch-perf-steady.windows-rtx4090.v1.json`, and `ui-gallery-hover-layout-torture-steady.windows-rtx4090.v1.json` are covered; other checked-in baselines need intentional re-seeding if p50 must be checked in. |
| Gates use the correct machine baseline. | `tools/perf/diag_resize_probes_gate.py` and `.sh` choose Windows RTX 4090 or macOS baseline by host platform. | Covered for resize helpers; other gate helpers remain explicit-baseline by design. |
| Gates use normalization hooks. | Resize gate helpers and baseline selectors now apply `tooling-suite-prewarm-fonts.json` and `tooling-suite-prelude-reset-diagnostics.json` by default. | Covered for the updated helpers/selectors. |
| A short real gate proves the helper path works. | `target/fret-diag/codex-resize-gate-default-hooks-smoke/summary.json`: `ui-resize-probes`, attempts=1, repeat=1, PASS, Windows baseline selected, default hooks recorded. | Smoke covered, not a full formal gate. |
| Full formal gates are green after the helper changes. | `target/fret-diag/codex-resize-flex-patch-gate-r7-v2-headroom30/summary.json`: Windows `ui-resize-probes` v2 passed attempts=3 repeat=7 with `pass_attempts=3`. `target/fret-diag-code-editor-resize-probes-windows-rtx4090-v2-gate-r7/summary.json`: `ui-code-editor-resize-probes` v2 passed attempts=3 repeat=7 with `pass_attempts=2`. | Covered for both resize gates. |
| Zed/GPUI and egui comparison remains explicit. | `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md` plus the contract matrix reference pressure column. | Covered as a design map; still needs updates when new gaps close. |
| Churn reduction is evidence-led. | Perf log entries show measured view-cache harness virtualization, code editor resize attribution, and decisions not to start broad root-solve quantization or `WindowedRowsSurface` rewrites without evidence. | Covered for recent work. |
| Pointer-move hit-test torture remains runnable as a contract surface. | `apps/fret-ui-gallery/src/ui/previews/pages/harness/hit_test_torture.rs` restores the `ui-gallery-hit-test-torture-root` surface; `tools/diag-scripts/suites/perf-ui-gallery-hit-test-torture-steady/suite.json` promotes the via-nav script; `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-smoke-r7/1778623477502/bundle.schema2.json` passed `--max-pointer-move-hit-test-us 100` and `--max-pointer-move-global-changes 0`; follow-up attribution in `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-attrib-r6/1778634174688/bundle.schema2.json` identified `dispatch_context_build_time_us` as the pointer dispatch tail; repeat=7 dispatch gate `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-gate-r9-repeat7/check.perf_thresholds.json` passed dispatch/hit-test/global-change thresholds `250us/100us/0`. | Covered as a formal pointer-move dispatch + hit-test gate for the optimized snapshot-cache path. |
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
- Linux smoke export:
  - `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.linux-local.v1.json` is a smoke-level
    `linux-local` export from the successful Linux GL bundle. It records `threshold_surface=ui`,
    `repeat=1`, and max-only values, so it is useful evidence but not a checked-in Linux
    editor-grade contract.
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
- Hit-test torture named suite recovery:
  - Restored page implementation:
    `apps/fret-ui-gallery/src/ui/previews/pages/harness/hit_test_torture.rs`.
  - Suite manifest:
    `tools/diag-scripts/suites/perf-ui-gallery-hit-test-torture-steady/suite.json`.
  - Gate smoke:
    `target/debug/fretboard-dev.exe diag perf perf-ui-gallery-hit-test-torture-steady --dir target/fret-diag/perf-ui-gallery-hit-test-torture-steady-smoke-r7 --repeat 1 --warmup-frames 5 --timeout-ms 300000 --sort hit_test --top 5 --json --reuse-launch --max-pointer-move-hit-test-us 100 --max-pointer-move-global-changes 0 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=20000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery.exe`
  - Result:
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-smoke-r7/1778623477502/bundle.schema2.json`
    passed the hit-test/global-change gate with `pointer_move_max_hit_test_time_us=17`,
    `pointer_move_snapshots_with_global_changes=0`, and bounds-tree queries/hits=`3/3`.
  - Separate finding:
    an exploratory `--max-pointer-move-dispatch-us 800` run failed at `1010us`. Dedicated follow-up
    attribution in
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-attrib-r6/1778634174688/bundle.schema2.json`
    shows p50/p95/max dispatch attribution `accounted=913/1139/1139us`,
    `unattributed=11/45/45us`, `runtime_wrapper=0/1/1us`, with the top frame dominated by
    `dispatch_context_build_time_us=1046us`. Keep the hit-test recovery gate unchanged and treat
    dispatch context/snapshot reuse as the next optimization target.
- Renderer payload contract surface:
  - `renderer_instance_bytes` and `renderer_encode_scene_text_ops` now flow through perf JSON rows, repeat summaries,
    baseline rows, `perf-baseline-from-bundles`, baseline parsing, threshold rows, and threshold failures.
  - `tools/perf/audit_perf_baselines.py --strict` now enforces the payload contract closure for payload-aware
    threshold surfaces by requiring the payload metrics in `measured_p50`, `measured_p90`, `measured_p95`,
    `measured_max`, `threshold_seed`, and the corresponding hard threshold fields. Unit coverage lives in
    `tools/perf/test_audit_perf_baselines.py`.
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
- IMUI hello semantic smoke gate:
  - Script: `tools/diag-scripts/ui-editor/imui/imui-hello-demo-semantic-smoke.json`
  - Suite: `tools/diag-scripts/suites/imui-hello-semantic-smoke/suite.json`
  - Command:
    `FRET_DIAG=1 FRET_DIAG_GPU_SCREENSHOTS=1 target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-editor/imui/imui-hello-demo-semantic-smoke.json --dir target/fret-diag/imui-hello-demo-semantic-smoke-pixels-r1 --session-auto --timeout-ms 180000 --check-pixels-changed imui-hello-demo.count-text --launch -- target/debug/imui_hello_demo.exe`
  - Suite command:
    `FRET_DIAG=1 FRET_DIAG_GPU_SCREENSHOTS=1 target/debug/fretboard-dev.exe diag suite imui-hello-semantic-smoke --dir target/fret-diag/imui-hello-semantic-smoke-suite-r2 --session-auto --timeout-ms 180000 --launch -- target/debug/imui_hello_demo.exe`
  - Result:
    `target/fret-diag/imui-hello-demo-semantic-smoke-pixels-r1/sessions/1778619037159-98320/script.result.json`
    passed at `step_index=20`.
  - Suite result:
    `target/fret-diag/imui-hello-semantic-smoke-suite-r2/sessions/1778619813105-103420/suite.summary.json`
    passed with `status=passed`, `wants_screenshots=true`, and one passed script row.
  - Pixel check:
    `target/fret-diag/imui-hello-demo-semantic-smoke-pixels-r1/sessions/1778619037159-98320/check.pixels_changed.json`
    resolved `imui-hello-demo.count-text` before/after count-change screenshots and saw the region hash change from
    `0x878210d4ffe36972` to `0xd1384d303356d837`.
  - Suite pixel check:
    `target/fret-diag/imui-hello-semantic-smoke-suite-r2/sessions/1778619813105-103420/check.pixels_changed.json`
    was produced without passing `--check-pixels-changed` explicitly. The `imui-hello-semantic-smoke` suite profile
    supplies the default `imui-hello-demo.count-text` check and saw the same before/after hash change from
    `0x878210d4ffe36972` to `0xd1384d303356d837`.
  - The script now machine-checks `Count: 0`, `Increment`, `Enabled: false`, unchecked checkbox state, then clicks
    `Increment` and captures a before/after screenshot pair for the count text region before clicking `Enabled` and
    waiting for checked state and `Enabled: true`.
  - Do not use `first_frame_smoke_demo` as text evidence: it intentionally paints only a full-window quad for runner
    bootstrap / first-present validation, so no text there is expected.
- Hit-test torture dispatch attribution:
  - `diag stats` now reports derived dispatch attribution fields (`dispatch_accounted_time_us` and
    `dispatch_unattributed_time_us`) in text and JSON output, plus follow-up body/wrapper/context
    attribution (`dispatch_inner_body_time_us`, `dispatch_runtime_wrapper_time_us`,
    `dispatch_context_build_time_us`).
  - Validation:
    `cargo nextest run -p fret-diag bundle_stats_reports_dispatch_unattributed_time --no-fail-fast`;
    `cargo test -p fret-diag bundle_stats_reports_dispatch_unattributed_time --no-fail-fast`;
    `cargo run -p fretboard-dev -- diag stats target/fret-diag/perf-ui-gallery-hit-test-torture-steady-smoke-r6/1778623403891/bundle.schema2.json --sort dispatch --top 1`.
  - Evidence on
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-smoke-r6/1778623403891/bundle.schema2.json`:
    dispatch attribution p50/p95/max `accounted=56/64/64us`, `unattributed=840/946/946us`; top dispatch frame
    `tick=229 frame=229` reports `dispatch_breakdown.us(total/accounted/unattributed/...)=1010/64/946/...`.
  - Follow-up evidence on
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-attrib-r6/1778634174688/bundle.schema2.json`:
    dispatch attribution p50/p95/max `accounted=913/1139/1139us`, `unattributed=11/45/45us`,
    `body_unattributed=11/45/45us`, and `runtime_wrapper=0/1/1us`. The top dispatch frame reports
    `context_build=1046us` with `hit_test=18us`.
  - Conclusion: the hit-test contract remains healthy, and the pointer dispatch tail is now attributed to dispatch
    context/snapshot construction. The next slice should investigate snapshot reuse or lazy focus snapshot
    construction before changing dispatch thresholds.
- Hit-test torture dispatch snapshot cache:
  - Implemented a retained-tree/layer-topology dispatch snapshot cache in `fret-ui`, with shared `Arc` snapshot
    forests for `nodes`, `parent`, `pre`, and `post`.
  - Validation:
    `cargo check -p fret-ui`;
    `cargo nextest run -p fret-ui dispatch_snapshot_cache_reuses_forest_across_frames_until_structure_changes --no-fail-fast`;
    `cargo nextest run -p fret-ui -E "test(~focus_scope) | test(~outside_press) | test(~window_input_arbitration_snapshot) | test(~window_command_action_availability_snapshot)" --no-fail-fast`;
    `cargo build -p fretboard-dev --release`;
    `cargo build -p fret-ui-gallery --release --features gallery-dev`;
    `target/release/fretboard-dev.exe diag perf perf-ui-gallery-hit-test-torture-steady --dir target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-snapshot-cache-r7 --repeat 1 --warmup-frames 5 --timeout-ms 300000 --sort dispatch --top 5 --json --reuse-launch --max-pointer-move-hit-test-us 100 --max-pointer-move-global-changes 0 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=20000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery.exe`.
  - Evidence on
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-snapshot-cache-r7/1778636234419/bundle.schema2.json`:
    pointer max dispatch/hit-test=`97/17us`, `pointer_move_snapshots_with_global_changes=0`;
    dispatch attribution p50/p95/max `accounted=79/91/91us`, `unattributed=3/6/6us`,
    `body_unattributed=2/5/5us`, `runtime_wrapper=0/1/1us`.
  - Repeat=3 evidence on
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-snapshot-cache-r8-repeat3/1778636608073/bundle.schema2.json`:
    `pointer_move_max_dispatch_time_us` min/p50/p95/max=`82/89/91/91`,
    `pointer_move_max_hit_test_time_us` min/p50/p95/max=`14/16/17/17`, and
    `pointer_move_snapshots_with_global_changes` min/p50/p95/max=`0/0/0/0`.
  - Repeat=7 formal gate:
    `target/release/fretboard-dev.exe diag perf perf-ui-gallery-hit-test-torture-steady --dir target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-gate-r9-repeat7 --repeat 7 --warmup-frames 5 --timeout-ms 300000 --sort dispatch --top 5 --json --reuse-launch --max-pointer-move-dispatch-us 250 --max-pointer-move-hit-test-us 100 --max-pointer-move-global-changes 0 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=20000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery.exe`
  - Helper entrypoint:
    `python tools/perf/diag_hit_test_torture_dispatch_gate.py --repeat 7`.
  - Repeat=7 threshold report:
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-gate-r9-repeat7/check.perf_thresholds.json`
    has `failures=[]` with thresholds dispatch/hit-test/global-change=`250us/100us/0`.
  - Worst repeat=7 bundle:
    `target/fret-diag/perf-ui-gallery-hit-test-torture-steady-dispatch-gate-r9-repeat7/1778636886432/bundle.schema2.json`.
  - Repeat=7 evidence:
    `pointer_move_max_dispatch_time_us` min/p50/p95/max=`79/87/112/112`,
    `pointer_move_max_hit_test_time_us` min/p50/p95/max=`13/16/20/20`, and
    `pointer_move_snapshots_with_global_changes` min/p50/p95/max=`0/0/0/0`.
  - Worst-bundle `diag stats --sort dispatch --top 5` reports dispatch/hit-test p50/p95=`86/112us` and
    `16/17us`, derived pointer max dispatch/hit-test=`112/17us`, dispatch attribution
    `accounted=79/105/105us`, `unattributed=4/7/7us`, and top-frame `context_build=3us`.
  - Conclusion: the prior `~1ms` pointer dispatch tail is fixed and now protected by a formal repeat=7
    dispatch-tail threshold gate.
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
- Editor Canvas replay formal evidence pass:
  - The 2026-05-16 repeat=3/warmup=5 macOS M4 evidence pass covers typical autoscroll, complex wheel, and resize
    jitter with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` and the standard prewarm/prelude hooks. See perf log entry
    `2026-05-16 01:03:00 +08:00`.
  - Worst bundles:
    `target/fret-diag/editor-canvas-replay-contract-evidence-20260516-typical-r3/1778862709522/bundle.schema2.json`,
    `target/fret-diag/editor-canvas-replay-contract-evidence-20260516-complex-wheel-r3/1778862752553/bundle.schema2.json`,
    and
    `target/fret-diag/editor-canvas-replay-contract-evidence-20260516-resize-jitter-r3/1778862785344/bundle.schema2.json`.
  - Attribution result: row replay/cache is healthy (`100%`, `99%`, `100%` hit rates; stores `0`, p95 `1`, `0`),
    while frame-level `paint.widget` remains `439..634us` p95 and renderer text prepare remains `419..435us`
    p95/max with atlas upload/eviction still `0`.
  - Conclusion: the next owner split is Canvas wrapper overhead versus renderer text/encode payload. Do not use this
    evidence to justify a broad `WindowedRowsSurface` display-list rewrite.
- Renderer text prepare reversible optimization:
  - `TextSystem::collect_scene_pinned_keys(...)` now pre-sizes glyph pin buckets from the scene's per-shape pin-key
    counts before merging text blobs. Code anchors:
    `crates/fret-render-wgpu/src/text/atlas_flow.rs` and `crates/fret-render-wgpu/src/text/atlas.rs`.
  - Validation: `cargo fmt -p fret-render-wgpu --check`; `cargo nextest run -p fret-render-wgpu --lib
    glyph_pin_keys_deduplicate_by_bucket glyph_key_buckets_with_capacities_deduplicate_by_bucket --no-fail-fast`;
    `cargo check -p fret-render-wgpu`; strict baseline audit passed.
  - After evidence: typical renderer text p95/max `360/376us`, complex wheel `381/412us`, and resize jitter
    `379/379us`, versus the formal pre-change `392/422us`, `412/435us`, and `419/419us`.
  - Contract decision: no baseline was updated or loosened from the macOS repeat=3 evidence. Existing payload-aware
    contracts remain the stabilization surface until a deliberate re-seed is justified.
- Windowed surface paint attribution fields:
  - `WindowedRowsSurface` now has an opt-in paint diagnostics hook, and code editor paint snapshots record surface
    callback, hook, row-loop, row-paint, non-row, and row-callback-gap counters when
    `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`.
  - Code anchors:
    `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`,
    `ecosystem/fret-code-editor/src/editor/diagnostics.rs`,
    `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`, and
    `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`.
  - Validation: focused `fret-ui-kit`, `fret-code-editor`, and `fret-diag` nextest slices passed; `cargo check -p
    fret-ui-gallery --features gallery-dev` passed. See perf log entry `2026-05-16 01:30:00 +08:00`.
  - Formal attribution evidence: the 2026-05-16 repeat=3/warmup=5 editor replay pass with these fields covers
    typical autoscroll, complex wheel, and resize jitter. Worst bundles:
    `target/fret-diag/editor-canvas-wrapper-attribution-20260516-typical-r3/1778865865185/bundle.schema2.json`,
    `target/fret-diag/editor-canvas-wrapper-attribution-20260516-complex-wheel-r3/1778865994148/bundle.schema2.json`,
    and
    `target/fret-diag/editor-canvas-wrapper-attribution-20260516-resize-jitter-r3/1778866025069/bundle.schema2.json`.
  - Attribution result: p95 `paint.widget` / surface callback / code-editor paint are `431/268/106us`
    typical, `653/489/321us` complex wheel, and `465/288/133us` resize jitter. Surface non-row p95 is
    `145/154/137us`, and row callback gap p95 is `21/14/23us`.
  - Contract decision: the inner `WindowedRowsSurface` attribution is now measurable and does not justify a broad
    display-list rewrite. The remaining outer `paint.widget - surface_callback` gap is still about `155..177us`
    p95, so the next optimization owner should be generic Canvas / `ElementHostWidget` paint bookkeeping.
- Paint-widget hotspot summary follow-up:
  - `fretboard diag stats --json` now emits `paint_widget_hotspot_summary`, sampling the top 16 paint-widget
    hotspots per frame and splitting Canvas from non-Canvas classes. Focused coverage:
    `cargo nextest run -p fret-diag bundle_stats_summarizes_canvas_paint_widget_hotspots --no-fail-fast`.
  - On the same formal bundles, Canvas hotspot p95 is `270/491/292us` while `WindowedRowsSurface` callback p95 is
    `268/489/288us`, so the single Canvas hotspot is effectively the surface callback, not a separate wrapper tax.
    Sampled top-N non-Canvas exclusive sum p95 is only `71/67/71us`.
  - Refined owner decision: the residual `paint.widget` cost after Canvas plus sampled top-N non-Canvas work is about
    `90..102us` p95. The next reversible slice should inspect generic `ElementHostWidget` / paint traversal aggregate
    overhead before any editor row replay or windowed-surface display-list rewrite.
- Host-widget paint subphase summary follow-up:
  - Existing snapshot fields for `paint_host_widget_observed_models_time_us`,
    `paint_host_widget_observed_globals_time_us`, `paint_host_widget_instance_lookup_time_us`, and their item/call
    counts are now promoted to root-level `diag stats` `p50`, `p95`, and `max` output.
  - On the same formal bundles, p95 host models/globals/lookup are `29/28/47us` typical, `29/29/47us` complex wheel,
    and `28/27/45us` resize jitter. This matches the scale of the remaining `paint.widget` residual and makes
    `ElementHostWidget::paint_impl` observed-dependency replay plus instance-record lookup the next narrow owner.
- Host-widget record lookup slimming:
  - `ElementHostWidget::paint_impl` now extracts only the paint-needed record fields instead of cloning the full
    retained element record.
  - Validation: `cargo fmt -p fret-ui --check`; `cargo check -p fret-ui`;
    `cargo nextest run -p fret-ui -E 'test(~paint)' --no-fail-fast`.
  - Exploratory no-reuse repeat=3 evidence:
    `target/fret-diag/editor-host-record-slim-20260516-typical-r3-noreuse`,
    `target/fret-diag/editor-host-record-slim-20260516-complex-wheel-r3-noreuse`, and
    `target/fret-diag/editor-host-record-slim-20260516-resize-jitter-r3-noreuse`.
    Host lookup p95 lands around `39..43us`, down from the earlier same-mouth formal `45..47us` range.
  - Contract decision: keep baselines unchanged because the comparable `--reuse-launch` repeat=3 formal run timed out
    after navigation state drift. The next contract-quality step is to stabilize that evidence path or define an
    explicit no-reuse formal policy before any re-seed.
- Editor paint reuse-launch evidence path recovery:
  - The timeout root cause was stale gallery nav search state under a reused process. At timeout, the app had not
    selected a page, `nav_query_len_bytes=37`, and the filtered nav list had no visible items while step 10 waited
    for `ui-gallery-nav-code-editor-torture`.
  - `ui-gallery-code-editor-torture-autoscroll-steady` and
    `ui-gallery-code-editor-window-resize-drag-jitter-steady` now use `type_text_into` with
    `clear_before_type=true` for the gallery nav search, matching the already-stable complex wheel probe.
  - Validation: JSON parse checks for both edited scripts, `python3 tools/check_diag_scripts_registry.py`, and
    `cargo nextest run -p fret-diag-protocol --no-fail-fast`.
  - Same-mouth formal evidence is restored with `--reuse-launch --repeat 3 --warmup-frames 5`:
    `target/fret-diag/editor-paint-contract-formal-20260516-typical-r3`,
    `target/fret-diag/editor-paint-contract-formal-20260516-complex-wheel-r3`, and
    `target/fret-diag/editor-paint-contract-formal-20260516-resize-jitter-r3`.
  - Results: typical total/paint p95 `807/572us`, complex wheel `1077/915us`, and resize jitter `1599/648us`.
    Worst-bundle row replay/cache remains healthy (`289/0`, `288/3`, `289/0` replay/store p95-equivalent rows),
    Canvas hotspot p95 tracks `WindowedRowsSurface` callback p95 within `1..4us`, and renderer text/encode top values
    stay in the `321..357us` / `149..190us` range without atlas upload/eviction pressure.
  - Contract decision: do not update baselines from this macOS M4 evidence. It restores the formal evidence path and
    confirms the next owner remains generic `ElementHostWidget` paint aggregate overhead.

## Editor Canvas Replay Goal Completion Audit (2026-05-16)

Goal under audit: close the Editor Canvas paint/cache replay to renderer-payload evidence loop, land reversible
optimizations only where attribution points, and stabilize the result as a durable perf contract.

| Requirement | Concrete evidence | Status |
| --- | --- | --- |
| Formal evidence pass covers typical autoscroll, complex wheel, and resize jitter with repeat/warmup and no repeat=1-only conclusions. | Perf log entries `2026-05-16 01:03:00 +08:00`, `2026-05-16 02:31:15 +08:00`, `2026-05-16 03:51:42 +08:00`, `2026-05-16 04:54:35 +08:00`, and `2026-05-16 05:14:52 +08:00`; overlay-disabled worst bundles under `target/fret-diag/editor-paint-overlay-disabled-20260516-*`. | Covered for macOS M4 evidence. Not sufficient for Windows baseline promotion. |
| Attribution identifies whether the remaining cost belongs to editor row replay/cache, generic Canvas/paint traversal, or renderer text/encode/upload. | Row replay/cache remains healthy; renderer text prepare got one reversible owner slice; `WindowedRowsSurface` attribution, paint-widget hotspot summary, host-widget subphase summary, and per-row gap fields are now in `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`. | Covered enough to reject broad row/display-list rewrites. Remaining owner is outer paint traversal / host-widget aggregate unless target evidence changes. |
| At least one reversible optimization is landed and verified without weakening renderer payload thresholds. | Renderer glyph pin-key bucket sizing, host-widget record lookup slimming, and observed-deps empty fast path were validated with focused checks; baselines stayed unchanged from local macOS evidence. | Covered locally; keep all slices rollbackable and baseline-neutral until target re-seed. |
| Contract stabilization promotes or validates the final thresholds on the target machine profile. | Current checked-in Windows contracts exist for resize, editor resize, autoscroll steady, autoscroll typical, and complex wheel; `audit_perf_baselines.py --strict` passes for the matrix. The target-machine runner is now `tools/perf/diag_editor_paint_contract_validate.py`; synced artifacts can be rechecked with `tools/perf/diag_editor_paint_contract_verify_artifacts.py`; the local closeout gate is `tools/perf/diag_editor_paint_contract_closeout.py`. The validation runner rejects existing non-empty non-dry-run output directories by default, preflight checks the required overlay-disabled script env defaults, the verifier requires non-empty `date_tag` fields, rejects drift from the required Windows command shape, rejects paint-perf env on baseline-validation direct `diag perf` commands, requires paint-perf coverage and overlay-zero stats (`top_code_editor_torture_overlay_us=0` / `code_editor_paint_perf.max.us_torture_overlay=0`) in the attribution directory, and the closeout CLI has unit coverage for both verifier-fail and verifier-pass non-dry-run paths. | Not complete. Needs deliberate Windows RTX4090 overlay-disabled validation with `check.perf_thresholds.json` `failures=[]`; only a threshold re-seed path additionally needs `selection-summary.json` plus the no-silent-threshold-loosening evidence. |
| P1.5 Editor Canvas paint replay can be closed. | Current audit, runbook, TODO, artifact verifier, and closeout gate now identify the remaining missing artifact, the target-machine runner command, the post-sync verification step, and the non-dry-run closeout behavior. | Not complete until target-machine validation, or a deliberate re-seed if validation fails, is recorded, verified, and the matrix/TODO are updated. |

Local handoff audit update (2026-05-16 14:46 +0800): the current macOS M4 workspace cannot close the Windows RTX4090
contract. `diag_editor_paint_contract_validate.py` rejects non-dry-run Windows-profile execution on this host, dry-run
directories fail the artifact verifier because they intentionally lack real `summary.json` files, and only the
artifact-independent local closeout gates are green. Clean Windows handoff plans live under
`target/fret-diag/editor-paint-contract-windows-handoff-*-plan*`; they are execution aids, not closeout evidence.

Active goal prompt-to-artifact checklist (2026-05-16): the closeout objective remains incomplete against actual
artifacts, not just intent.

| Objective requirement | Required artifact or command | Current evidence | Status |
| --- | --- | --- | --- |
| Run `tools/perf/diag_editor_paint_contract_validate.py` and produce a baseline validation artifact. | A non-dry-run Windows RTX4090 validation directory with `summary.json` kind `editor_paint_contract_validate_summary`, `with_paint_perf=false`, `ok=true`, and step threshold reports with `failures=[]`. | `python3 tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260516-completion-audit-host-guard` rejects this macOS arm64 host with `windows-rtx4090 validation must run on the target Windows host`; no matching validation `summary.json` exists under current `target/fret-diag/editor-paint-contract*` directories. | Missing. |
| Run the same target-machine validation with `--with-paint-perf` and produce attribution artifacts. | A second non-dry-run Windows RTX4090 validation directory with `summary.json`, `with_paint_perf=true`, paint-perf coverage, and overlay-zero stats. | Only dry-run attribution plans exist locally. The artifact verifier report `target/fret-diag/editor-paint-contract-validate-20260516-completion-audit/artifact-verification.dry-run-negative.summary.json` fails because `target/fret-diag/editor-paint-contract-validate-20260516-closeout-plan-attrib/summary.json` is missing. | Missing. |
| Confirm artifacts with the verifier and closeout tools. | `diag_editor_paint_contract_verify_artifacts.py` and `diag_editor_paint_contract_closeout.py` pass on the copied validation and attribution directories. | The verifier is green only for tool behavior; against current dry-run plan directories it correctly fails with missing validation and attribution summaries. The local preflight passes 8 checks but does not cover target-machine perf results. | Missing. |
| Decide whether the next refactor lane is Canvas/paint replay, glyph/text-index/atlas residency, or no code change. | Decision based on verified target-machine baseline and attribution artifacts. | macOS M4 evidence points away from Canvas wrapper and row replay and toward generic host-widget paint aggregate, but target-machine attribution is still absent. | Not decidable from the required evidence. |

## Open Gaps

1. The broad `ui-gallery-steady` suite remains evidence-only until it is redefined as a suite-of-contracts or its
   membership is intentionally narrowed. Its former broad-only members are now covered by dedicated Windows contracts;
   do not try to re-promote the broad suite by loosening thresholds.
2. The autoscroll typical v2 and complex wheel v1 contracts cover stricter editor paint/payload surfaces, the complex
   wheel overlay hotspot now has a narrower frame-derived-state fix, and the 2026-05-16 formal macOS evidence pass
   shows healthy row-scene replay/cache. Keep the `WindowedRowsSurface` display-list rewrite gated on a future
   near-threshold or failing stressor where row op replay/capture is the measured limiter, not on these passing
   baselines alone.
3. The first renderer owner slice has landed as a reversible glyph pin-bucket capacity optimization, the
   `WindowedRowsSurface` attribution fields are wired through app snapshots and `fretboard diag stats`, the
   paint-widget hotspot summary now proves Canvas hotspot p95 tracks the surface callback p95 within `1..4us`, and
   root-level host-widget paint subphase summaries identify observed-dependency replay plus instance-record lookup
   as the next owner. The first narrow lookup optimization is landed, the script-level reuse-launch drift that
   blocked same-mouth formal evidence has been fixed, and the observed-deps presence-set fast path now makes the
   empty lookup case explicit (`244/252` empty calls in the typical autoscroll evidence bundle). The post-fast-path
   three-probe formal macOS M4 pass is also green:
   `target/fret-diag/editor-paint-contract-post-observed-deps-fastpath-20260516-typical-r3-cargo`,
   `target/fret-diag/editor-paint-contract-post-observed-deps-fastpath-20260516-complex-wheel-r3-cargo`, and
   `target/fret-diag/editor-paint-contract-post-observed-deps-fastpath-20260516-resize-jitter-r3-cargo`. That pass is
   still evidence-only; a baseline decision requires a deliberate re-seed rather than using local machine evidence to
   loosen or silently update thresholds.
4. Keep Linux and any other non-Windows/macOS machine profiles explicit until a real Linux runner/profile and checked-in contract baseline exist. The current `ui-code-editor-resize-probes.linux-local.v1.json` export is smoke-only and does not close the gap.
5. The current WSL code-editor resize smoke gate still times out on the current head after rebuild, with
   `Connection reset by peer` in `stderr.log` and `stage=running` at `step_index=5`; do not infer a
   checked-in Linux editor-grade baseline from this run.
6. Formal editor torture probes now default `FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0` in the contract scripts.
   The overlay-disabled repeat=3 pass reports `top_code_editor_torture_overlay_us=0` for typical autoscroll, complex
   wheel, and resize jitter:
   `target/fret-diag/editor-paint-overlay-disabled-20260516-typical-r3/1778878430806/bundle.schema2.json`,
   `target/fret-diag/editor-paint-overlay-disabled-20260516-complex-wheel-r3/1778878778260/bundle.schema2.json`,
   and `target/fret-diag/editor-paint-overlay-disabled-20260516-resize-jitter-r3/1778878807245/bundle.schema2.json`.
   That keeps the diagnostic HUD available for manual debugging while removing it from the measured contract. This is
   still macOS M4 evidence only; do not re-seed or loosen checked-in baselines from it.
## Audit Conclusion

The goal is not complete. The Windows `ui-resize-probes` and `ui-code-editor-resize-probes` contracts now have
checked-in `measured_p50` evidence and green formal repeat=7 gates, the code-editor autoscroll steady, autoscroll
typical, and complex wheel contracts now have payload-aware baselines with explicit UI threshold modes, and
`ui-gallery-view-cache-toggle-perf-steady`, `ui-gallery-virtual-list-torture-steady`, `ui-gallery-menubar-keyboard-nav-steady`,
`ui-gallery-material3-tabs-switch-perf-steady`, and `ui-gallery-hover-layout-torture-steady` are now dedicated Windows
v1 contracts. The hit-test torture pointer-move path now also has a formal repeat=7 dispatch/hit-test threshold gate
for the optimized dispatch snapshot cache path. The 2026-05-16 Editor Canvas replay evidence has now closed one
renderer-side owner slice without loosening baselines, the `WindowedRowsSurface` paint attribution fields have been
verified on formal bundles, and `paint_widget_hotspot_summary` narrows the remaining editor paint owner to generic
paint-widget aggregate overhead rather than Canvas wrapper, renderer payload, or code-editor row replay. Root-level
host-widget subphase summaries now make that owner measurable, the first lookup-slimming slice is directionally
positive, and the observed-deps presence-set fast path identifies and short-circuits the dominant empty lookup case.
The same-mouth editor paint evidence path has been restored and the full three-probe post-fast-path pass is complete:
typical total/paint p95 `850/624us`, complex wheel `1115/838us`, and resize jitter `1563/631us`. Keep baselines
unchanged from this macOS M4 evidence. Follow-up attribution now shows the Canvas wrapper itself is not the owner:
Canvas-minus-`WindowedRowsSurface` callback p95 is only `2..4us`, while callback-minus-row-paint p95 is
`118..149us`. The subsequent per-row inspection keeps the remaining surface gap in aggregate loop-overhead territory
rather than a standalone row hot loop. Keep non-Windows machine profiles explicit rather than inferring them from the
Windows RTX 4090 contract set.
The 2026-05-16 05:14 +08:00 `diag stats` follow-up added per-row derived fields and shows the three overlay-disabled
editor bundles at `65/62/62ns` per row for `windowed_surface_paint_callback_minus_row_paint_per_row_ns` and
`79/48/72ns` per row for `windowed_surface_row_callback_gap_per_row_ns` (typical autoscroll, complex wheel, and resize
jitter respectively). That keeps the remaining surface gap in aggregate loop overhead territory rather than a standalone
row hot loop. The next blocking step is target-machine validation with
`python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date>`. Local independent optimization slices may
continue when they have their own evidence, but they are not substitutes for this target-machine closeout and must not
drive checked-in baseline changes unless the Windows RTX4090 artifacts justify it.
