# ADR 0195: `delinea` Interaction + Hit Testing Contract (AxisPointer/Tooltip/Legend v1)

Status: Proposed

## Context

ECharts-class charts feel coherent because interaction semantics are consistent across chart types:

- hover/tooltip policies (`trigger=item` vs `trigger=axis`),
- axis pointers (crosshair/line/shadow),
- legend selection and highlight,
- zoom/pan/lock behavior.

In Fret, we want interaction semantics to be deterministic and testable. That implies that the headless
engine (`delinea`) owns the interaction model, and the UI adapter (`fret-chart`) is responsible for:

- capturing input events,
- mapping them to headless actions,
- rendering the headless outputs (crosshair lines, tooltip panels, legend UI).

## Relationship to Other ADRs

- ADR 0190: headless engine boundary.
- ADR 0191: transform pipeline, `dataZoom` ordering, axis pointer baseline.
- ADR 0192: axis mapping contract (required for correct hit testing and axis-trigger tooltip).
- ADR 0193: stable identity and marks output contract.
- ADR 0098: plot input mapping (ImPlot-like); chart interaction is higher-level and dataset-driven.

## Decision

### 1) Interaction state lives in the headless model, not in widget-local ad hoc structs

The chart model includes interaction-affecting state:

- axis windows (dataZoom / locks),
- legend visibility (selected series),
- axis pointer state (hover position, trigger mode),
- selection/brush state (future).

The UI adapter maintains only ephemeral UI concerns (animation timers, text measurement caches).

### 2) Hit testing returns structured results keyed by stable identity

Hit testing must be stable across filtering/reordering and must never rely on “series index”.

Contract:

- Hover results are keyed by `SeriesId` and optional “item identity” (index or interpolated sample).
- `trigger=item` requires a hit (nearest segment/point within a threshold).
- `trigger=axis` produces a value per visible series when sampling is possible, even if no curve
  is within a hit threshold.

`HoverHit.y_value` is the **rendered** Y value in data space for the hovered series. For stacked series,
this means `y_value` includes the stack base (the same value used for coordinate mapping), so the tooltip
and hit testing stay consistent with what is drawn.

#### v1 tooltip defaults (ECharts-aligned)

To match common “charting app” expectations (and avoid surprising “nothing happens” states), v1 defaults are:

- `AxisPointerSpec.trigger = Axis`
- `AxisPointerSpec.snap = false` (opt-in)

Semantics:

- `trigger=Axis` always shows the axis pointer/crosshair when inside the plot rect.
- With `snap=true`, the axis pointer may snap to the nearest “meaningful” item when a close-enough hit exists:
  - line-family: nearest segment, with sampling at the snapped X.
  - scatter: nearest point (no interpolation).
- When no hit exists, the pointer uses the raw cursor X for axis sampling (line-family interpolation allowed).
- Axis-trigger tooltips are stable and complete by default:
  - The first line is the trigger axis label/value.
  - Then one line per visible series in `series_order`.
  - If a series cannot be sampled at the current axis value (missing/NaN/out of range), its value is `-`.

### 3) Input -> action mapping is explicit and portable

The UI adapter maps platform input (pointer, wheel, keys) into headless actions.
The headless layer defines the **meaning** of actions.

v1 action set (minimum):

- set axis pointer position (in plot-local coordinates),
- set series visibility (legend toggle),
- pan/zoom X window (dataZoom inside behavior),
- reset view.

Keyboard shortcut choices are UI-policy, but the action semantics are headless.

### 4) Axis lock and zoom lock are first-class axis interaction policies (P0)

To avoid ad hoc behavior drift, each axis may define:

- pan enabled/disabled,
- zoom enabled/disabled,
- lock min/max (already supported as `AxisRange` forms),
- optional “keep aspect / lock ratio” (future, for synchronized axes).

This is required to implement “axis lock” and “zoom lock” parity features without scattering
conditionals across UI code.

#### Contract surface (v1)

Headless state:

- `ChartState.axis_locks: BTreeMap<AxisId, AxisInteractionLocks>`
- `AxisInteractionLocks { pan_locked: bool, zoom_locked: bool }`

Headless actions:

- `Action::ToggleAxisPanLock { axis }`
- `Action::ToggleAxisZoomLock { axis }`
- `Action::PanDataWindowXFromBase { axis, base, delta_px, viewport_span_px }`
- `Action::PanDataWindowYFromBase { axis, base, delta_px, viewport_span_px }`
- `Action::ZoomDataWindowXFromBase { axis, base, center_px, log2_scale, viewport_span_px }`
- `Action::ZoomDataWindowYFromBase { axis, base, center_px, log2_scale, viewport_span_px }`
- `Action::SetDataWindowXFromZoom { axis, base, window, anchor }`
- `Action::SetDataWindowYFromZoom { axis, base, window, anchor }`

Semantics:

1. If the axis is `AxisRange::Fixed`, the corresponding pan/zoom actions are no-ops.
2. If `pan_locked` is true, pan actions are no-ops.
3. If `zoom_locked` is true, zoom actions are no-ops.
4. Otherwise, resulting windows are clamped with `AxisRange::LockMin/LockMax` constraints.

## Consequences

- Interaction becomes more consistent across desktop and wasm.
- We can unit-test hover/tooltip behavior in `delinea` without rendering.
- Future components (brush selection, dataZoom slider) become incremental additions instead of rewrites.

## Follow-ups

P0:

- Refactor any remaining widget-local interaction state into the `delinea` model when it affects semantics.
- Add unit tests for axis lock + zoom lock policies once implemented.

P1:

- Add multi-axis and multi-grid interaction policies (axis pointer sync, per-grid zoom).
- Add brush selection (rectangle/lasso) as a headless component.

## References

- ADR 0190: `docs/adr/0190-delinea-headless-chart-engine.md`
- ADR 0191: `docs/adr/0191-delinea-transform-pipeline-and-datazoom-semantics.md`
- ECharts axisPointer/tooltip concepts: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\echarts.all.ts`
