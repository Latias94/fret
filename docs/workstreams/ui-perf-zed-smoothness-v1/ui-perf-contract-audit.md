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
| Representative editor-grade scripts are listed. | `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md` maps steady gallery, resize, code editor resize, view-cache resize torture, pointer move/hit-test, and renderer/effects churn. | Covered: the matrix now classifies the view-cache resize torture scripts under `ui-resize-probes` instead of leaving them as an unresolved side surface. |
| Baseline rows can record `p50/p95/max`. | `crates/fret-diag/src/diag_perf/baseline_rows.rs` writes `measured_p50`; smoke output `target/fret-diag/codex-p50-baseline-smoke/baseline.json` included `measured_p50`. | Tooling covered for new baselines. |
| Renderer payload metrics can become hard contract fields. | `crates/fret-diag/src/diag_perf/stats_rows.rs`, `runs_rows.rs`, `reporting.rs`, `baseline_rows.rs`, `thresholds.rs`, `crates/fret-diag/src/compare.rs`, and `crates/fret-diag/src/diag_perf_baseline.rs` now propagate `renderer_instance_bytes` and `renderer_encode_scene_text_ops` through perf JSON, baseline JSON, baseline parsing, threshold rows, and threshold failures. The code-editor autoscroll v4 baseline is the first checked-in `ui-renderer-payload` contract. | Covered for the autoscroll editor paint contract; older baselines that predate these fields remain valid time-only contracts until intentionally re-seeded. |
| Checked-in baselines actually contain `p50`. | Scan on 2026-05-11 after the Windows resize/code-editor v2 promotions, autoscroll steady v4 promotion, autoscroll typical v2 promotion, view-cache toggle v1 promotion, virtual-list v1 promotion, menubar keyboard nav v1 promotion, Material 3 tabs v1 promotion, and hover-layout v1 promotion: 78 baseline files, 314 max-bearing perf rows, 18 rows with `measured_p50`. | Partially covered. `ui-resize-probes.windows-rtx4090.v2.json`, `ui-code-editor-resize-probes.windows-rtx4090.v2.json`, `ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v4.json`, `ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json`, `ui-gallery-view-cache-toggle-perf-steady.windows-rtx4090.v1.json`, `ui-gallery-virtual-list-torture-steady.windows-rtx4090.v1.json`, `ui-gallery-menubar-keyboard-nav-steady.windows-rtx4090.v1.json`, `ui-gallery-material3-tabs-switch-perf-steady.windows-rtx4090.v1.json`, and `ui-gallery-hover-layout-torture-steady.windows-rtx4090.v1.json` are covered; other checked-in baselines need intentional re-seeding if p50 must be checked in. |
| Gates use the correct machine baseline. | `tools/perf/diag_resize_probes_gate.py` and `.sh` choose Windows RTX 4090 or macOS baseline by host platform. | Covered for resize helpers; other gate helpers remain explicit-baseline by design. |
| Gates use normalization hooks. | Resize gate helpers and baseline selectors now apply `tooling-suite-prewarm-fonts.json` and `tooling-suite-prelude-reset-diagnostics.json` by default. | Covered for the updated helpers/selectors. |
| A short real gate proves the helper path works. | `target/fret-diag/codex-resize-gate-default-hooks-smoke/summary.json`: `ui-resize-probes`, attempts=1, repeat=1, PASS, Windows baseline selected, default hooks recorded. | Smoke covered, not a full formal gate. |
| Full formal gates are green after the helper changes. | `target/fret-diag/codex-resize-flex-patch-gate-r7-v2-headroom30/summary.json`: Windows `ui-resize-probes` v2 passed attempts=3 repeat=7 with `pass_attempts=3`. `target/fret-diag-code-editor-resize-probes-windows-rtx4090-v2-gate-r7/summary.json`: `ui-code-editor-resize-probes` v2 passed attempts=3 repeat=7 with `pass_attempts=2`. | Covered for both resize gates. |
| Zed/GPUI and egui comparison remains explicit. | `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md` plus the contract matrix reference pressure column. | Covered as a design map; still needs updates when new gaps close. |
| Churn reduction is evidence-led. | Perf log entries show measured view-cache harness virtualization, code editor resize attribution, and decisions not to start broad root-solve quantization or `WindowedRowsSurface` rewrites without evidence. | Covered for recent work. |
| Baseline maintenance rules are documented. | `docs/workstreams/perf-baselines/README.md` defines machine tags, re-seed criteria, required hooks, selector workflow, validation workflow, and review checklist. | Covered. |
| Completion criteria are unambiguous. | This audit maps requirements to evidence and gaps. | Covered, with open gaps below. |

## Current Evidence Snapshot

- Recent commits:
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
  - `BASELINE_FILES=78`
  - `TOTAL_ROWS=314`
  - `TOTAL_ROWS_WITH_P50=18`
  - `TOTAL_ROWS_WITH_P90=163`
  - `TOTAL_ROWS_WITH_P95=163`
  - `TOTAL_ROWS_WITH_MAX=314`
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
    `cargo run -q -p fretboard -- diag stats target/fret-diag-baseline-select-ui-gallery-hover-layout-torture-steady-windows-rtx4090-v1-policy/candidate-2-baseline/1778476920836/bundle.schema2.json --check-hover-layout-max 0`
    passed; `hover.decl_inv(layout/hit/paint)=0/0/0`.
- Renderer payload contract surface:
  - `renderer_instance_bytes` and `renderer_encode_scene_text_ops` now flow through perf JSON rows, repeat summaries,
    baseline rows, `perf-baseline-from-bundles`, baseline parsing, threshold rows, and threshold failures.
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

## Open Gaps

1. The broad `ui-gallery-steady` suite remains evidence-only until it is redefined as a suite-of-contracts or its
   membership is intentionally narrowed. Its former broad-only members are now covered by dedicated Windows contracts;
   do not try to re-promote the broad suite by loosening thresholds.
2. The autoscroll typical v2 contract covers a stricter typical-frame editor paint/payload surface, but it is not a
   high-stress scroll/edit/resize paint contract. Keep the `WindowedRowsSurface` display-list rewrite gated on a
   future near-threshold or failing stressor, not on this passing baseline alone.
3. Keep non-Windows/macOS machine profiles explicit until a checked-in baseline and owner profile exist.

## Audit Conclusion

The goal is not complete. The Windows `ui-resize-probes` and `ui-code-editor-resize-probes` contracts now have
checked-in `measured_p50` evidence and green formal repeat=7 gates, the code-editor autoscroll steady and typical
contracts now have payload-aware baselines, and `ui-gallery-view-cache-toggle-perf-steady`,
`ui-gallery-virtual-list-torture-steady`, `ui-gallery-menubar-keyboard-nav-steady`,
`ui-gallery-material3-tabs-switch-perf-steady`, and `ui-gallery-hover-layout-torture-steady` are now dedicated Windows
v1 contracts. The next work should only start a `WindowedRowsSurface` display-list rewrite from a near-threshold or
failing editor paint stressor, and keep non-Windows machine profiles explicit rather than inferring them from the
Windows RTX 4090 contract set.
