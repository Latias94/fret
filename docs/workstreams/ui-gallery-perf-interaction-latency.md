---
title: UI Gallery Performance (Interaction Latency)
status: draft
date: 2026-02-01
scope: ui-gallery, perf, interaction-latency
---

# UI Gallery Performance (Interaction Latency)

This workstream tracks **interaction latency** reports in `apps/fret-ui-gallery` where a user action (typically a click)
has a noticeable delay before the UI responds (e.g. “~0.5s until the right panel shows”).

This is separate from scroll-specific investigations (see `docs/workstreams/ui-gallery-perf-scroll-measure.md`).

## Goals

- Convert “feels slow” into a **deterministic perf repro** (`tools/diag-scripts/ui-gallery-*.json`) that isolates the
  post-input frame budget (ideally after `reset_diagnostics`).
- Make `fretboard diag perf` reports **name the hot subtree** via `SemanticsProps.test_id` anchors.
- Produce a short evidence bundle that identifies the primary cost center(s): layout solve, text shaping, paint, cache
  invalidation, or model churn.

## Non-goals

- Prematurely optimizing every page in UI Gallery.
- Rewriting the caching model without an ADR / contract change.

## Baseline workflow

1. Add a script that:
   - starts on the target page (`FRET_UI_GALLERY_START_PAGE=...`),
   - optionally `reset_diagnostics`,
   - performs the interaction (e.g. click card),
   - waits a few frames for the response,
   - captures a bundle.
2. Run perf:

```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/<script>.json `
  --env FRET_UI_GALLERY_START_PAGE=<page> `
  --warmup-frames 0 --sort time --top 1 --json `
  --launch -- cargo run -p fret-ui-gallery --release
```

3. Inspect the worst bundle and identify the dominant category:

```powershell
cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 1 --json
```

## Repro: Sidebar “Card” click

Script:

- `tools/diag-scripts/ui-gallery-nav-card-click-latency.json`

Run (debug build):

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-nav-card-click-latency.json `
  --env FRET_UI_GALLERY_START_PAGE=button `
  --launch -- cargo run -p fret-ui-gallery
```

Then summarize the “second click” bundle (look for `*-ui-gallery-nav-card-click-latency-second/bundle.json`):

```powershell
cargo run -p fretboard -- diag stats target/fret-diag/<dir>/bundle.json --sort time --top 1
```

Notes:

- Prefer `diag run` + `diag stats` for interaction latency triage: `diag perf` can attribute work to the click step that
  injected input, but the slow frame may be the *next* render/present.
- When the UI “feels stuck”, check `frame delta (ms)` in `diag stats` output. A large `dt_ms` on the top row indicates a
  long delay between frames (not just an expensive frame).

## Findings (2026-02-01)

### Summary

In **debug builds**, the worst post-click frame is dominated by **scrollable content measurement** for the UI Gallery
content viewport (`SemanticsProps.test_id = ui-gallery-content-viewport`). This matches the reported 0.5s+ latency:
the frame is blocked inside a deep `measure()` walk while the ScrollArea probes its content extent.

In **release builds**, the same path is still the top cost but is typically ~100ms-scale, not ~900ms-scale.

### Evidence (debug build)

Bundle: the `*-ui-gallery-nav-card-click-latency-second/bundle.json` captured by the script.

Key signals in the worst snapshot (often `frame_id ~= 74`):

- `debug.stats.layout_time_us ~= 950k`
- `debug.stats.layout_roots_time_us ~= 940k`
- `debug.stats.layout_engine_solve_time_us ~= single-digit ms`
- `debug.widget_measure_hotspots[0]`:
  - `widget_type = fret_ui::declarative::host_widget::ElementHostWidget`
  - `inclusive_time_us ~= 900k`
  - the corresponding `debug.semantics.nodes[]` entry has `test_id = ui-gallery-content-viewport`

Interpretation:

- This is **not** dominated by layout engine solve time (taffy time is small).
- The dominant cost is a deep `measure()` subtree under the ScrollArea viewport required to compute the scrollable
  content extent when `ScrollProps.probe_unbounded = true` (default).
- `viewport_intrinsic_measure_mode = Viewport` helps intrinsic sizing, but it does not change this layout-time probe.

### Immediate mitigation options

- Prefer `--release` for UI Gallery perception checks (debug will exaggerate stalls for deep measure walks).
- Consider `profile.dev` opt-level overrides for hot crates (`fret-ui`, `fret-ui-shadcn`, `fret-render`) to reduce
  interactive latency during development without switching to full release builds.

## Notes

- If the perf report cannot attribute time to a meaningful region, add minimal `test_id` anchors at the demo surface
  (UI Gallery first) or the component layer (shadcn) so `diag stats` nested children become actionable.
