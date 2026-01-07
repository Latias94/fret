# Plot Axis Interactions (ImPlot Alignment Notes)

This document describes the axis-region interaction policy in `fret-plot` and how it aligns with
the baseline UX of ImPlot. It complements `docs/plot-implot-alignment.md` and focuses on the parts
that often feel "off" in editor-grade UIs: axis-only pan/zoom, fit behavior, and axis locks.

## Regions

`fret-plot` treats the plot widget as a set of interaction regions:

- Plot region: the data viewport.
- X axis region: the horizontal axis strip below the plot.
- Y axis regions: the left axis strip plus the optional right-side Y2/Y3/Y4 strips.

Region hit testing is based on the computed `PlotLayout` rectangles and is stable across scale
factors.

## Wheel zoom policy

Wheel zoom is routed based on region:

- Wheel over plot: zoom X and Y together (with axis-only modifiers enabled by default).
- Wheel over X axis: zoom X only.
- Wheel over a Y axis: zoom that Y axis only (left/Y2/Y3/Y4 depending on the region).

Axis-only modifiers when the wheel is over the plot region:

- `Shift`: zoom X only (default; configurable via `PlotInputMap::wheel_zoom_x_only_mod`)
- `Ctrl`: zoom Y only (default; configurable via `PlotInputMap::wheel_zoom_y_only_mod`)

Note: ImPlot defaults do not include axis-only modifiers for the wheel, but we keep them because
they are common in editor UIs and work well with axis locks.

### Wheel zoom input mapping (PlotInputMap)

The wheel zoom implementation uses `PlotInputMap` to decide when zoom is allowed and how it is
constrained when the pointer is over the plot region.

| InputMap field | Default | Effect |
| --- | --- | --- |
| `wheel_zoom_mod` | `None` | When set, wheel zoom only activates if the modifier is pressed. |
| `wheel_zoom_x_only_mod` | `Some(Shift)` | Over plot region: zoom X only (Y zoom factor forced to 1.0). |
| `wheel_zoom_y_only_mod` | `Some(Ctrl)` | Over plot region: zoom Y only (X zoom factor forced to 1.0). |

Precedence rules (over the plot region):

- If `wheel_zoom_mod` is set and not pressed, wheel zoom is ignored.
- If `wheel_zoom_x_only_mod` matches, it takes precedence over `wheel_zoom_y_only_mod`.
- Otherwise if `wheel_zoom_y_only_mod` matches, apply Y-only zoom.

Over axis regions, the axis region routing takes precedence (X axis always X-only, Y axis always
that axis only).

## Drag pan policy

Pan is routed based on the region where the drag begins:

- Drag in plot region: pan X and all visible Y axes together.
- Drag in X axis region: pan X only.
- Drag in a Y axis region: pan that Y axis only.

Pan is “lazy-started”: pressing the pan chord does not immediately change view state. The view is
only updated after a pointer move event, which preserves double-click detection for fit actions.

## Fit policy (double-click)

Fit (default: `LMB` double-click) is routed by region:

- Double-click in plot: fit X + all visible Y axes.
- Double-click on X axis: fit X only (preserving current Y ranges).
- Double-click on a Y axis: fit that Y axis only (preserving X and other Y ranges).

If an axis is zoom-locked, fit does not change that axis.

## Axis locks

Axis locks are split into two independent flags:

- Pan lock: prevents view changes caused by pan gestures.
- Zoom lock: prevents view changes caused by wheel zoom, box-zoom, and fit on that axis.

Locks apply consistently across plot-region and axis-region interactions.

Lock state is stored in `PlotState::axis_locks`, so it can be persisted or shared when the caller
provides an external `Model<PlotState>`.

### UI & shortcuts

The goal is to keep locks discoverable without forcing a context menu early:

- `Ctrl+Click` on an axis region toggles both pan+zoom lock for that axis.
- `L` toggles both pan+zoom lock for the region under the pointer.
- `Shift+L` toggles pan lock for the region under the pointer.
- `Ctrl+L` toggles zoom lock for the region under the pointer.

These shortcuts are configurable via `PlotInputMap`:

- `axis_lock_click`
- `axis_lock_toggle`
- `axis_pan_lock_toggle`
- `axis_zoom_lock_toggle`

When `L`-toggling in the plot region, `fret-plot` treats this as a “global lock” toggle and
applies it to X and all visible Y axes.
