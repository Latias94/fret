# ADR 0198: `delinea` DataZoom Y + 2D Semantics (ECharts-Inspired)

Status: Accepted (P0)

## Context

`delinea` already supports:

- X dataZoom semantics with `FilterMode` (ADR 0191),
- per-axis view windows (`DataWindowX` / `DataWindowY`) and durable constraints (`AxisRange`) with clear precedence,
- multi-axis layout and interaction targeting via `fret-chart` (ADR 0196),
- pan/zoom locks (ADR 0197).

The remaining high-impact gap for ECharts-class cartesian charts is zooming beyond “X-only”:

- Y-only zoom/pan (wheel/drag on Y axis band, and plot-region modifiers),
- 2D box zoom / brush zoom (write both X and Y windows as a single interaction),
- consistent composition with `FilterMode`, `AxisRange`, and multi-axis routing,
- performance constraints for large datasets (avoid allocations and avoid sparse selections in v1).

Apache ECharts supports `dataZoom` for multiple dimensions and a set of filter behaviors
(`filter`, `weakFilter`, `empty`, `none`). Importantly, ECharts can filter across dimensions, which
often implies sparse selections or per-dimension masking (expensive on large data).

This ADR locks down the minimum semantics we need for Y and 2D zoom **without forcing a v1 redesign**
of our transform pipeline or selection representation.

## Relationship to Other ADRs

- ADR 0191: transform pipeline and X `FilterMode` semantics.
- ADR 0192: axis scales + coordinate mapping.
- ADR 0194: large data + progressive rendering strategy.
- ADR 0196: multi-axis + UI layout contract (active axis routing).
- ADR 0197: axis locks and shortcut policy.
- `ecosystem/delinea/docs/axis-range-and-window-policy.md`: current precedence rules and window math.

## Decision

### 1) Y zoom is mapping-only in v1 (no row filtering by Y)

In v1, Y zoom/pan changes the **effective Y mapping window** but does not change the per-series row
selection.

Concretely:

- Y zoom updates `ChartState.data_window_y[axis]` (or clears it).
- The transform pipeline continues to derive row selection **only from X** (dataset row ranges + X window),
  as described in ADR 0191 and `axis-range-and-window-policy.md`.

This keeps v1 allocation-free and avoids introducing a sparse selection type prematurely.

Implications:

- Y zoom does not hide points by filtering them out of the dataset selection; it only changes projection.
- LOD and marks generation may clamp/intersect Y values against the effective Y window for stability, but
  they do not skip rows purely based on Y.

### 2) `FilterMode` remains X-only in v1

We keep `FilterMode` as an X-window policy:

- `FilterMode::Filter`: slice rows to the X window (fast on monotonic X).
- `FilterMode::None`: do not slice rows to the X window (global bounds/LOD).

No Y `filterMode` is introduced in v1. This avoids confusing “filter” semantics that cannot be implemented
consistently without sparse selections and per-series masking.

### 3) 2D zoom is expressed as a paired view-window write

2D zoom/brush is represented as a single action that writes two 1D windows:

- `Action::SetViewWindow2D { x_axis, y_axis, x, y }`
- `Action::SetViewWindow2DFromZoom { x_axis, y_axis, base_x, base_y, x, y }` (interaction-derived box zoom)

Semantics:

- Both windows are updated in the same revision, so the engine produces marks based on a coherent 2D view.
- `AxisRange` constraints are applied to both windows (and may override/lock bounds).
- Pan/zoom locks gate whether the action is applied per axis (ADR 0197).

Row selection behavior:

- Even in 2D zoom, row selection remains X-driven, controlled by X `FilterMode`.
- Y is never used to filter/slice rows in v1.

### 4) Multi-axis routing: 2D operations target the active axis pair

When multiple X and/or Y axes exist in a grid:

- UI adapters (notably `fret-chart`) choose an “active axis pair” as defined in ADR 0196.
- Plot-region 2D interactions (box zoom / reset) apply to the active pair.
- Axis-band interactions apply to the hovered axis for that dimension and the active other axis when
  an interaction requires both.

This keeps behavior deterministic and discoverable without adding ECharts-style axisIndex arrays in v1.

### 5) `AxisRange` precedence remains unchanged

We keep the existing precedence rules:

- `AxisRange::Fixed` overrides view windows for that axis.
- Partial locks (`LockMin` / `LockMax`) constrain the resulting window but do not fully override it.
- Constraints apply on every write to view windows (including `SetViewWindow2D` and `SetViewWindow2DFromZoom`).

### 6) Represent “durable dataZoom configuration” separately from view state

ECharts has a durable `dataZoom` component model with options like `minSpan`, `maxSpan`, `rangeMode`,
`realtime`, and axis bindings.

In `delinea` we continue to separate:

- **Durable options** in `ChartSpec` / `ChartModel` (serializable chart configuration).
- **Ephemeral view windows** in `ChartState` (interactive state).

v1 scope:

- X: `DataZoomXSpec` remains the durable home of X `filterMode` defaults.
- Y: `DataZoomYSpec` exists as a durable placeholder for future persisted defaults and slider UI, but v1
  interactions can rely on
  `ChartState.data_window_y` without requiring a durable component.

## Consequences

- Y and 2D zoom can be implemented without introducing sparse selections or per-dimension masking in v1.
- Large-data behavior remains predictable: row slicing stays O(log n) for monotonic X series.
- Semantics remain compatible with the “ECharts-inspired” mental model while explicitly documenting where
  we diverge for performance and simplicity.

## Follow-ups

P1:

- Decide whether Y needs its own `FilterMode`, and whether `DataZoomYSpec` should grow durable defaults (`rangeMode`, persisted windows).
- Introduce `RowSelection` variants beyond `Range` (e.g. indices/bitmap) so future “filter by Y” and
  ECharts-like `weakFilter`/`empty` become feasible without ad-hoc allocations.
- Add span constraints (`minSpan/maxSpan` or `minValueSpan/maxValueSpan`) as a durable policy key,
  and define how they compose with `AxisRange` and locks.

## Amendments

- 2026-01-13 (ECharts replica workstream): `DataZoomYSpec` gained an opt-in `filter_mode` to explore
  ECharts-style multi-dimensional filtering. The default remains mapping-only (`filter_mode=None`).
  The current v1 subset materializes sparse indices selections only for non-stacked cartesian series
  (Line/Area/Band/Scatter) under view-size caps, and expresses `Empty` as a typed view-level mask
  (`SeriesEmptyMask`) consumed consistently by marks + axisPointer/tooltip (see ADR 0211).

## Evidence

- Action surface: `ecosystem/delinea/src/action/mod.rs` (`ZoomDataWindowYFromBase`, `PanDataWindowYFromBase`, `SetViewWindow2DFromZoom`)
- Engine semantics: `ecosystem/delinea/src/engine/mod.rs` (lock gating + window writes for Y and 2D actions)
- Budgeted indices carriers for filtering/masks: `ecosystem/delinea/src/transform_graph/data_view.rs`
- View-level `Empty` participation contract: `ecosystem/delinea/src/view/mod.rs` (`SeriesEmptyMask`)
- UI adapter (multi-axis routing + 2D box zoom): `ecosystem/fret-chart/src/retained/canvas.rs`
- Conformance harness (desktop + wasm): `apps/fret-examples/src/chart_multi_axis_demo.rs`
