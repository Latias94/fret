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
- `docs/workstreams/perf-baselines/ui-gallery-complex-steady.windows-rtx4090.v1.json` (tail / spikes, `top_*`)
- `docs/workstreams/perf-baselines/ui-gallery-complex-typical.windows-rtx4090.v1.json` (typical perf, `frame_p95_*`)

Seed policy (how thresholds were derived):

- `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json`
- `docs/workstreams/perf-baselines/policies/ui-gallery-complex-typical.v1.json`

## P0 runbook (fast gate check)

Prebuild (once):

- `cargo build -p fretboard-dev -p fret-ui-gallery --release`

Recommended env (avoid extra I/O + keep cached rendering on):

- `FRET_DIAG_SCRIPT_AUTO_DUMP=0`
- `FRET_DIAG_SEMANTICS=0`
- `FRET_UI_GALLERY_VIEW_CACHE=1`
- `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`
- `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`

P0 commands:

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 3 --warmup-frames 5 --reuse-launch --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env ... --launch -- target/release/fret-ui-gallery.exe`
- `python tools/perf/diag_resize_probes_gate.py --suite ui-resize-probes --attempts 3 --repeat 7 --baseline docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v2.json --launch-bin target/release/fret-ui-gallery.exe`
- `python tools/perf/diag_resize_probes_gate.py --suite ui-code-editor-resize-probes --attempts 3 --repeat 7 --baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v2.json --launch-bin target/release/fret-ui-gallery.exe`

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
- `top_paint_time_us` / `frame_max_paint_time_us`: paint-tail or renderer churn. Inspect renderer stage timings and trace.
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
  - When a threshold fails, `failures[]` includes `actual_p95_us`, `outlier_suspected`, and `evidence_bundle` (a bundle.json path you can feed to `diag stats`) for quick triage.

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
- If you want a **typical-perf gate**, create a dedicated baseline seeded from percentiles and then
  gate using `--perf-threshold-agg p95`.

Example (local typical baseline; does not change the canonical baselines):

- Create a p95-seeded baseline:
  - `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 15 --warmup-frames 5 --perf-baseline-out .fret/perf.baseline.p95.json --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json --perf-baseline-seed this-suite@top_total_time_us=p95 --launch -- target/release/fret-ui-gallery.exe`
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

## Failure exemplar map

- Layout-root build spikes: `Finding (2026-02-14): repeat=7 can fail on Material3 tabs (request_build_roots dominates)`.
- Layout-engine solve spikes: `Finding (2026-02-15): Batch-solve barrier roots to eliminate per-root solve spikes`.
- Paint spikes: `Finding (2026-05-10): ui-gallery-complex-steady now yields a paint-dominant Windows exemplar when run with --prelude-each-run; use target/fret-diag/1778364986668/bundle.schema2.json for paint-tail attribution.`

## Next steps

### 1) Reduce remaining tail spikes (Windows-specific)

Hypotheses to validate:

- allocator jitter (large transient allocations outside the frame arena)
- hash/vec capacity growth on “rare” paths
- background thread wakeups competing with the UI thread during resize

Candidate actions (small → large):

- tighten capacity reuse for known hot scratch structures (avoid occasional rehash/grow)
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
