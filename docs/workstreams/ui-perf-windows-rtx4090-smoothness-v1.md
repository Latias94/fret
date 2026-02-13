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
- `FRET_DIAG_SCRIPT_AUTO_DUMP=0`
- `FRET_DIAG_SEMANTICS=0`

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

