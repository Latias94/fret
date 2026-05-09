# UI Performance Contract Matrix (Zed Smoothness v1)

Status: Draft workstream contract.

This matrix is the quick way to compare Fret's editor-grade performance probes with the current checked-in baselines,
gate commands, recent evidence, and the Zed/GPUI plus egui reference concern each probe is meant to protect.

Related:

- Workstream: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1.md`
- TODO tracker: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-todo.md`
- Perf log: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- GPUI/egui gap map: `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md`
- Baselines: `docs/workstreams/perf-baselines/`
- Baseline maintenance: `docs/workstreams/perf-baselines/README.md`
- Contract audit: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-audit.md`

## Contract Rules

- Representative scripts should report `p50`, `p95`, and `max` in evidence notes.
- New `diag perf --perf-baseline-out` files record row-level `measured_p50`, `measured_p90`, `measured_p95`, and
  `measured_max`.
- Existing checked-in baselines created before `measured_p50` was added remain valid. Do not synthesize p50 into old
  files; add it only by intentionally re-seeding a baseline on the target machine.
- Use environment-specific baselines. `diag_resize_probes_gate.py` and `.sh` choose the checked-in Windows RTX 4090 or
  macOS baseline by host platform; pass `--baseline` explicitly for another machine profile.
- Resize gate helpers apply the default font prewarm and reset-diagnostics prelude hooks. Use
  `--no-default-suite-hooks` only when intentionally debugging setup behavior.
- Baseline `threshold_surface` must match the suite intent. Resize/layout baselines use `ui` so renderer micro timings
  remain attribution evidence under `measured_*` instead of noisy hard thresholds; renderer/effects suites can opt into
  `renderer` or `all`.

## Target Budgets

| Tier | Intent | Representative CPU budget |
| --- | --- | --- |
| Tier A | 60Hz baseline | `p95 total <= 8ms`, `max total <= 16ms` |
| Tier B | 120Hz / Zed feel | `p95 total <= 4ms`, `max total <= 8ms` |

Budgets are guidance for representative probes. The committed gate is the script-specific baseline plus headroom.

## Matrix

| Probe group | Representative script or suite | Checked-in baseline | Gate command | Latest evidence | Reference pressure |
| --- | --- | --- | --- | --- | --- |
| Canonical steady gallery | `ui-gallery-steady` | `docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json`; macOS history under `ui-gallery-steady.macos-m4.v*.json` | `cargo run -p fretboard --release -- diag perf ui-gallery-steady --repeat 7 --warmup-frames 5 --reuse-launch --suite-prewarm tools/diag-scripts/tooling-suite-prewarm-fonts.json --suite-prelude tools/diag-scripts/tooling-suite-prelude-reset-diagnostics.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --perf-baseline <baseline> --launch -- cargo run -p fret-ui-gallery --release` | Perf log daily smoke entries `2026-05-07 13:58` and `2026-05-07 14:01` | GPUI cached views, egui repaint/pass accounting |
| Resize probes | `ui-resize-probes` | `docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v2.json`; `docs/workstreams/perf-baselines/ui-resize-probes.macos-m4.v3.json` | `python tools/perf/diag_resize_probes_gate.py --suite ui-resize-probes --attempts 3 --repeat 7` | Perf log `2026-05-09 20:41`: Windows v2 records `measured_p50/p95/max`, uses `threshold_surface=ui`, and passed formal attempts=3 repeat=7 gate with 30% headroom | GPUI resize coalescing and bounded layout roots; egui bounded extra work |
| Code editor resize | `ui-code-editor-resize-probes`; `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json` | `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v1.json`; `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json` | `python tools/perf/diag_resize_probes_gate.py --suite ui-code-editor-resize-probes --attempts 3` | Perf log `2026-05-09 18:05`: repeat=3 p95 `total/layout/paint/solve=3995/2137/1747/574us`, 20k-line torture surface active | GPUI visible-window text reuse and idempotent render setters |
| View-cache resize torture | `tools/diag-scripts/ui-gallery/perf/ui-gallery-window-resize-stress-steady.json`; `tools/diag-scripts/ui-gallery/perf/ui-gallery-window-resize-drag-jitter-steady.json` | No dedicated post-virtualization baseline yet; covered indirectly by resize probes | Run single scripts with prewarm/prelude and record `p50/p95/max`; re-seed only after the suite membership is stable | Perf log `2026-05-09 17:52`: page-local view-cache root element count `1104 -> 137`; drag-jitter repeat=3 p95 `2066/1310/754/643us` | GPUI shrinks hot layout boundaries instead of hiding real width deltas |
| Pointer move / hit test | `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json` | Usually threshold-gated directly rather than baseline-selected | Use `--max-pointer-move-dispatch-us`, `--max-pointer-move-hit-test-us`, and `--max-pointer-move-global-changes` with `FRET_DIAG_SEMANTICS=0` | GPUI gap map section `0.3`; earlier perf log entries for commits `763bf8e7`, `8bc15eda`, `7fa76fd5`, `5ab4ba71` | GPUI bounds-tree hit testing; egui explicit repaint cause discipline |
| Renderer/effects churn | `tools/diag-scripts/ui-gallery-effects-blur-torture-steady.json`; `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`; SVG/clip/headless stress gates | `clip-mask-stress-headless.windows-local.v1.json`, `quad-material-stress-headless.windows-local.v1.json`, `svg-atlas-stress-headless.windows-local.v1.json`, effect-specific baselines | Use the corresponding `tools/perf/*_gate.py` helper for each renderer contract | GPUI gap map section `Gap D`; renderer telemetry entries in the perf log | GPU resource churn should be explainable, not hidden behind CPU frame time |

## Current Gaps

- Non-Windows/macOS resize gate runs still need an explicit `--baseline` until we add checked-in baselines for those
  machine profiles.
- Old baseline JSON files may not contain `measured_p50`. That is expected until each baseline is intentionally
  re-seeded.
- Windows `ui-resize-probes` v2 is now checked in with `threshold_surface=ui`, 30% headroom, and row-level
  `measured_p50/p90/p95/max`. Earlier 20% headroom attempts remained too tight for observed resize/layout tail
  variability; treat that as the current Windows contract rationale rather than a renderer threshold problem.
- There is no dedicated post-virtualization baseline for the view-cache resize torture scripts. Keep using them as
  evidence scripts until we decide whether they belong in `ui-resize-probes` or a narrower gallery-harness suite.
- Code editor resize is green on the current Windows RTX 4090 evidence. A larger `WindowedRowsSurface` paint rewrite
  needs a stricter editor paint stressor or a failing/near-threshold gate before it is justified.
