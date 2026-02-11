# ADR 0197: `delinea` Axis Interaction Locks and Shortcut Contract (P0)

Status: Accepted (P0)

## Context

`delinea` (headless) and `fret-chart` (UI adapter) support cartesian pan/zoom interactions. We also
need a stable, testable contract for:

- What it means to "lock" an axis (pan lock vs zoom lock).
- How locks interact with `AxisRange` constraints (`Auto`, `LockMin`, `LockMax`, `Fixed`).
- What default shortcuts/gestures are supported by `fret-chart`, and what they map to in the
  headless action surface.

This ADR is intentionally scoped to cartesian 2D and the current P0 interaction surface. It does
not attempt to exactly match ECharts UI components (e.g. `dataZoom.slider`) yet, but it keeps the
data model aligned with ECharts (`dataZoom` state, filter modes, and axis constraints).

## Goals

- Make axis locks explicit and deterministic (no hidden side effects).
- Ensure `AxisRange` constraints are consistently applied whenever view windows are written.
- Provide a single reference for default `fret-chart` gestures/shortcuts and their mapping.
- Keep the contract compatible with multi-axis layouts (ADR 0196).

## Non-Goals

- Implementing `dataZoom.slider`, Y/2D zoom components, or brush selection UI.
- Defining a public theming/formatting callback API for tooltips/ticks.
- Guaranteeing parity with ImPlot/ECharts for every shortcut (we document defaults, not mandates).

## Definitions

### AxisRange (spec-level constraint)

`AxisRange` constrains an axis in data space:

- `Auto`: unconstrained.
- `LockMin { min }`: lower bound is fixed; upper bound may vary.
- `LockMax { max }`: upper bound is fixed; lower bound may vary.
- `Fixed { min, max }`: both bounds fixed (fully overrides interactive view windows in v1).

### AxisInteractionLocks (runtime interaction gate)

`AxisInteractionLocks` is stored in `ChartState.axis_locks` per axis:

- `pan_locked`: prevents interactive panning for that axis.
- `zoom_locked`: prevents interactive zooming for that axis.

Locks are orthogonal to `AxisRange`. They do not change the data model; they only gate interaction
actions.

## Decisions

### 1) Locks gate *interaction* actions, not programmatic setters

Locks apply to:

- `Action::PanDataWindow*FromBase`
- `Action::ZoomDataWindow*FromBase`
- `Action::SetDataWindow*FromZoom` (box zoom output)

Locks do **not** apply to:

- `Action::SetDataWindowX`
- `Action::SetDataWindowY`
- `Action::SetViewWindow2D`

Rationale:
- UI gestures should respect user locks.
- Programmatic view changes (reset/fit/linking) must remain possible and predictable.

### 2) AxisRange constraints are applied on every view-window write

Whenever a view window is written into `ChartState` (including programmatic setters), the window is
clamped and `AxisRange.locked_min/locked_max` constraints are applied.

This ensures:
- The stored state is always coherent and respects spec-level constraints.
- UI and engine never diverge due to inconsistent clamping paths.

### 3) Partial locks preserve the current span when possible

`DataWindow::apply_constraints(locked_min, locked_max)` must produce a valid non-degenerate window.
For partial constraints (`LockMin` or `LockMax`), we preserve the existing span when the constraint
would otherwise invert the window.

Example:
- base window: `[0, 10]`
- apply `LockMin { min: 200 }` -> `[200, 210]`

### 4) Default `fret-chart` shortcuts map to stable headless actions

`fret-chart::retained::ChartCanvas` provides defaults aligned with ImPlot-style interactions, while
remaining compatible with ECharts semantics in the data model.

Defaults (see `ecosystem/fret-chart/src/input_map.rs`):

- Pan: LMB drag (plot region) -> `Action::PanDataWindow*FromBase` (per-axis gating + constraints).
- Box zoom: RMB drag (plot region) -> `Action::SetDataWindow*FromZoom`.
- Box zoom (alternative): Shift + LMB drag.
- Wheel zoom:
  - over plot: zoom X+Y
  - over X axis band: zoom X only
  - over Y axis band: zoom Y only
  - modifiers in plot: Ctrl disables X, Shift disables Y
  -> `Action::ZoomDataWindow*FromBase`
- Reset view:
  - double click axis band: reset that axis window (programmatic setter)
  - double click plot: reset both primary axes
  - `R`: reset view
- Fit view:
  - `F`: fit to data extents (programmatic setter + constraints)
- Toggle interaction locks:
  - `Ctrl + LMB` on an axis band: toggle pan+zoom locks for that axis.
  - `Ctrl + LMB` on plot: toggle pan+zoom locks for primary axes.
  - `L` (uses last pointer position to pick axis region):
    - `L`: toggle pan+zoom
    - `Shift + L`: toggle pan only
    - `Ctrl + L`: toggle zoom only

Notes:
- `AxisRange::Fixed` prevents panning/zooming regardless of interaction locks.
- Multi-axis layouts route interactions based on the axis band under the pointer (ADR 0196).

## Implementation Notes (Current)

- Engine lock state: `ecosystem/delinea/src/engine/interaction.rs`
- Headless action surface: `ecosystem/delinea/src/action/mod.rs`
- Lock + range enforcement:
  - `ChartEngine::apply_pan_from_base` / `apply_zoom_from_base` (interaction path)
  - `ChartEngine::apply_action` for `SetDataWindow*` / `SetViewWindow2D` (programmatic path)
- UI mapping: `ecosystem/fret-chart/src/retained/canvas.rs`

## Testing

P0 tests should cover:

- Pan/zoom actions are no-ops when the corresponding lock is enabled.
- `SetDataWindow*` and `SetViewWindow2D` apply `AxisRange` constraints.
- `DataWindow::apply_constraints` preserves span for partial locks.
