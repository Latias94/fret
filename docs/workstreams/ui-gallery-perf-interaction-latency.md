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

## Notes

- If the perf report cannot attribute time to a meaningful region, add minimal `test_id` anchors at the demo surface
  (UI Gallery first) or the component layer (shadcn) so `diag stats` nested children become actionable.

