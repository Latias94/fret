# ADR 0205: `delinea` Brush Selection + Interaction Output Contract (ECharts-Inspired)

Status: Accepted (P0)

## Context

Apache ECharts distinguishes between:

- **View/window controls** (e.g. `dataZoom`): they write axis windows and change what is visible.
- **Selections** (e.g. `brush`): they define a selection region and produce selection output/events, but do not
  inherently change the view window.

`delinea` already has durable, headless view/window semantics via:

- X `dataZoom` filtering/windowing (ADR 0191),
- Y view windows (mapping-first in v1; optional indices materialization is size-capped; ADR 0198 + ADR 0211),
- 2D box zoom as paired view-window writes (ADR 0198),
- selection/filtering contracts for large data (ADR 0199).

However, `fret-chart` currently implements a **UI-local** 2D brush rectangle that is not surfaced to the headless
engine. Without an explicit headless contract, future requirements (linking, external consumers, cross-chart brush,
streaming safety) risk forcing a large refactor.

This ADR locks a minimal, allocation-free baseline for brush selection that keeps v1 fast while leaving a clear
extension path to ECharts-class behavior (per-series selection, sparse indices, value masking).

## Relationship to Other ADRs

- ADR 0191: X `dataZoom` filtering semantics.
- ADR 0195: interaction/hit testing contract.
- ADR 0198: Y + 2D view semantics.
- ADR 0199: `RowSelection` and filtering contract (contiguous fast path).

## Decision

### 1) Brush selection is a first-class interaction output (not a view window write)

`delinea` treats brush selection as a separate concept from axis windows:

- Brush selection **does not** write X/Y view windows (that remains the responsibility of box zoom and `dataZoom`).
- Brush selection is stored in headless state and exposed in headless output so that apps can consume it without
  depending on a specific UI adapter.

### 2) v1 supports a single selection primitive: cartesian rectangle in data space

We introduce a v1 baseline selection shape:

- `BrushSelection2D`: a rectangular selection region expressed as a pair of data windows (X and Y) plus the targeted
  axis ids (multi-axis aware). For multi-grid charts, the selection also carries an optional `GridId` so downstream
  consumers can route without guessing.

This choice is:

- deterministic (data-space, not pixel-space),
- allocation-free to update during pointer drags,
- compatible with future conversion into `RowSelection` / per-series point masks.

### 3) Headless action surface

Brush selection updates are expressed as actions:

- `Action::SetBrushSelection2D { x_axis, y_axis, x, y }`
- `Action::ClearBrushSelection`

Notes:

- Brush actions are not gated by axis pan/zoom locks (ADR 0197); locks only gate view-window mutations.
- The UI adapter is responsible for gesture mapping (e.g. `Alt + RMB drag`) and for deciding which axis pair the
  selection targets (usually the active axis pair; ADR 0196).

### 4) State + output surface

Brush selection is stored in headless state and exposed in output:

- `ChartState.brush_selection_2d: Option<BrushSelection2D>`
- `ChartOutput.brush_selection_2d: Option<BrushSelection2D>`

This makes brush selection observable for:

- app-level selection linking,
- external consumers (exporting selections, driving inspectors),
- future highlight/masking pipelines.

In multi-grid charts, the engine must keep the selection scoped to a single grid. Implementations should validate
that `x_axis` and `y_axis` resolve to the same grid; mismatches clear the selection defensively.

### 5) Future extensions (P1+)

This ADR explicitly leaves space for ECharts-class behaviors:

- Conversion to `RowSelection` (range/indices) when the preconditions are satisfied
  (e.g. monotonic X for fast slicing; ADR 0199).
- Per-series or per-dataset brush selections (multiple selection groups).
- Value masking / "empty" semantics (line breaks without filtering) as a separate concept (ADR 0199).

## Consequences

- Brush selection becomes a durable headless output (no UI-only semantics).
- The v1 implementation stays minimal and allocation-free.
- Highlight/visual feedback remains a UI concern for now; a future ADR can define renderer-visible selection styling.

## References

- Apache ECharts brush component and selection output:
  - `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\component\\brush\\*`
- Selection/filter contract baseline: `docs/adr/0199-delinea-row-selection-and-filtering-contract.md`
