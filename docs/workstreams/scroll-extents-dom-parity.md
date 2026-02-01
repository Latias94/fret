---
title: Scroll Extents (DOM/GPUI Parity)
status: draft
date: 2026-02-01
scope: fret-ui, scroll, layout, perf
---

# Scroll Extents (DOM/GPUI Parity)

This workstream proposes a more scalable scroll extent strategy for `fret-ui` that avoids deep
`measure()` probes for large scrollable surfaces (notably UI Gallery page content, markdown/code
views, and other editor-grade panels).

For the current UI Gallery perf investigation context, see:

- `docs/workstreams/ui-gallery-perf-scroll-measure.md`

## Problem Statement

Today, `ScrollProps::probe_unbounded = true` drives a MaxContent-style probe on the scroll axis.
This often forces a deep subtree `measure()` walk to determine the scrollable content extent.

In debug/dev builds, this can cause noticeable stalls on page switches (e.g. UI Gallery nav click)
because the frame is blocked inside recursive measurement rather than only doing a single final
layout pass.

## Reference Direction (GPUI / DOM)

The DOM model (and GPUI’s implementation style) treats scroll extent as a property that can be
derived from **final layout geometry**, rather than requiring an additional “unbounded measure”
probe.

In particular, GPUI computes `content_size` from child layout bounds and then clamps the scroll
offset / computes `max_offset` after layout (see `repo-ref/zed/crates/gpui/src/elements/div.rs`).

This avoids performing a second deep subtree measurement solely to answer “how tall is the scroll
content?”

## Proposed Direction

Move `fret-ui` scroll extent computation toward “post-layout geometry”:

1. Layout children under the viewport-sized box (or otherwise well-defined container bounds).
2. Compute scroll extents from the resulting geometry (child bounds union, plus padding/border).
3. Clamp offsets and expose `max_offset` for scrollbars/automation.

Key constraints:

- Correctness first: no layout oscillation, stable anchor rects for overlays, deterministic scroll
  offset clamping across frames.
- Preserve an escape hatch for “true unbounded” width probing (e.g. code editor horizontal scroll)
  if needed, but avoid using it as the default for common vertical scrolling.

## Implementation Notes / Risks

- `probe_unbounded` currently couples two concerns:
  - how the subtree is measured (intrinsic sizing),
  - how scroll extent is derived.
  Untangling these is likely required.
- Some elements currently clamp measurement to the available size, which effectively forbids
  overflow unless a MaxContent probe is used. Moving toward a DOM-like model may require revisiting
  these constraints and/or introducing explicit “overflow allowed” semantics along the scroll axis.

## Next Steps

Track concrete tasks and “done criteria” in:

- `docs/workstreams/scroll-extents-dom-parity-todo.md`

