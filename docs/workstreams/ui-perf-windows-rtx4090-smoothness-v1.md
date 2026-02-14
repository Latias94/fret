# UI Perf (Windows RTX4090) — Smoothness v1

## Context

This workstream targets **editor-grade smoothness** (low tail latency, few spikes) on **Windows** for the
`windows-rtx4090` machine profile.

The goal is to turn “it feels janky sometimes” into a **repeatable, explainable, reversible** performance
contract:

- Gates should pass reliably (majority-of-attempts for tail stability).
- Worst bundles must be explainable via `fretboard diag stats`.
- Optimizations must be rollbackable (small, scoped changes with evidence).

## Goals (Acceptance)

- `ui-gallery-steady` passes against the canonical Windows baseline.
- `ui-resize-probes --attempts 3` passes a strict majority of attempts (>= 2/3).
- `ui-code-editor-resize-probes` does not regress.

## Measurement Protocol (P0)

Use `--launch` with release binaries and keep diagnostics overhead stable:

- `FRET_UI_GALLERY_VIEW_CACHE=1`
- `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`
- `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`
- `FRET_DIAG_SCRIPT_AUTO_DUMP=0`
- `FRET_DIAG_SEMANTICS=0`

Note: recent `fretboard diag perf` builds make the UI gallery cache knobs implicit for the built-in
`ui-gallery(-steady)` and resize probe suites (caller-overridable via `--env KEY=...`). The VirtualList
known-heights knob is only made implicit for `ui-gallery(-steady)`. Keep them explicit when bisecting
older `fretboard.exe` builds.

Gate triage loop:

1. Inspect `<outDir>/check.perf_thresholds.json`
2. Resolve the worst bundle (per failing script/metric).
3. Attribute via:
   - `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30`

## Known Hitch Class: System Font Rescan → `TextFontStackKey` Bump

One common tail-spike class on Windows came from **system font rescan results** being applied during
interactive resize workloads.

When the runner applies the rescan result, it bumps `fret_runtime::TextFontStackKey`. The UI tree treats
that key as a layout dependency, which can trigger large relayouts and view-cache disruption in the worst
frames.

### Mitigations landed (v1)

- Runner: briefly **defers applying** a completed rescan result while the window surface size is actively
  changing (resize window).
- Diag scripts: add a `wait_until` predicate for **`TextFontStackKey` stability** before `reset_diagnostics`
  in perf suites, so one-off font-stack churn is less likely to land inside a measured window.

## Next Steps

1. Re-run P0 gates (attempts=3) and record:
   - pass/fail per attempt,
   - worst bundles,
   - 3–5 load-bearing metrics (total/layout/solve/paint).
2. If `ui-gallery-steady` still flakes after font stability changes, triage by failing script:
   - `menubar-keyboard-nav-steady`,
   - `virtual-list-torture-steady`,
   - `view-cache-toggle-perf-steady`,
   and extract worst bundles for attribution.
3. If resize probes remain borderline, focus on reducing tail in:
   - `layout_time_us` (root-building vs solve),
   - text measurement churn (wrap-width bucketing, shaping cache behavior),
   - cache invalidation fan-out (global/model observation patterns).

## Evidence (2026-02-13)

Commit-addressable gate results for the `windows-rtx4090` machine profile:

- Changes:
  - `9816a115` — `perf(diag): stabilize perf windows on Windows`
  - `99b14370` — `perf(ui): stabilize interactive resize tails`
- Gates (baseline: `*.windows-rtx4090.v1.json`):
  - `ui-resize-probes --repeat 3`: PASS
    - out-dir: `target/fret-diag-perf/ui-resize-probes.hoverstrip.3x.20260213-151459`
  - `ui-code-editor-resize-probes --repeat 3`: PASS
    - out-dir: `target/fret-diag-perf/ui-code-editor-resize-probes.hoverstrip.3x.20260213-151711`
  - `ui-gallery-steady --repeat 3`: PASS
    - out-dir: `target/fret-diag-perf/ui-gallery-steady.hoverstrip.3x.20260213-152340`
  - `ui-gallery-steady --repeat 3` (VirtualList known heights): PASS
    - out-dir: `target/fret-diag-perf/ui-gallery-steady.hoverstrip.3x.20260213-183818.vlist-known`
  - `ui-resize-probes` P0 attempts=3 (`--repeat 3` each): PASS (3/3)
    - attempt 1 out-dir: `target/fret-diag-perf/ui-resize-probes.final.r3.20260213-204550`
      - worst bundle: `target/fret-diag-perf/ui-resize-probes.final.r3.20260213-204550/1770986783548-ui-gallery-window-resize-drag-jitter-steady/bundle.json` (total=29879us, layout=21265us, solve=6355us)
    - attempt 2 out-dir: `target/fret-diag-perf/ui-resize-probes.p0-attempt-2.r3.20260213-213034`
      - worst bundle: `target/fret-diag-perf/ui-resize-probes.p0-attempt-2.r3.20260213-213034/1770989476248-ui-gallery-window-resize-drag-jitter-steady/bundle.json` (total=31959us, layout=23231us, solve=6381us)
    - attempt 3 out-dir: `target/fret-diag-perf/ui-resize-probes.p0-attempt-3.r3.20260213-213137`
      - worst bundle: `target/fret-diag-perf/ui-resize-probes.p0-attempt-3.r3.20260213-213137/1770989521330-ui-gallery-window-resize-drag-jitter-steady/bundle.json` (total=30718us, layout=21583us, solve=6500us)
- Commands used (PowerShell, release):
  - `target/release/fretboard.exe diag perf ui-resize-probes --dir <out> --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v1.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- target/release/fret-ui-gallery`
  - `target/release/fretboard.exe diag perf ui-code-editor-resize-probes --dir <out> --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v1.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- target/release/fret-ui-gallery`
  - `target/release/fretboard.exe diag perf ui-gallery-steady --dir <out> --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- target/release/fret-ui-gallery`

## Evidence (2026-02-14)

Re-run P0 gates after merging latest `main` into the worktree branch (baseline: `*.windows-rtx4090.v1.json`):

- `ui-code-editor-resize-probes --repeat 3`: PASS
  - out-dir: `target/fret-diag-perf/ui-code-editor-resize-probes.p0.r3.20260214-114448`
- `ui-resize-probes --repeat 3`: PASS
  - out-dir: `target/fret-diag-perf/ui-resize-probes.p0.r3.20260214-114537`
- `ui-gallery-steady --repeat 3`: FAIL
  - out-dir: `target/fret-diag-perf/ui-gallery-steady.p0.r3.20260214-113753`
  - failures (baseline thresholds):
    - `ui-gallery-view-cache-toggle-perf-steady` (`top_total_time_us`, `top_layout_time_us`)
      - worst bundle: `target/fret-diag-perf/ui-gallery-steady.p0.r3.20260214-113753/1771040322789-ui-gallery-view-cache-toggle-perf-steady/bundle.json` (total=40131us, layout=30242us)
    - `ui-gallery-material3-tabs-switch-perf-steady` (`top_layout_time_us`, `top_layout_engine_solve_time_us`)
    - `ui-gallery-virtual-list-torture-steady` (`top_layout_engine_solve_time_us`)

Attribution notes:

- The view-cache toggle outlier is dominated by `layout_time_us` (not `solve_us`), and shows a large layout footprint
  (`layout.nodes=1120` in the top frame). This suggests “tree/layout invalidation work” rather than Taffy solve cost.
- Current triage hints do not trigger for this outlier (no `solve_heavy` / `observation_heavy`), indicating a need for
  additional layout sub-phase metrics (e.g. root-building) and/or new hints for cache-root invalidation churn.
