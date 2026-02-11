# ADR 0196: `delinea` Multi-Axis + Layout Contract (Cartesian v1)

Status: Accepted (P0)

## Context

To reach “ECharts-class” chart capability, we need to support:

- multiple X and/or Y axes in a single cartesian grid (e.g. left + right Y axes),
- deterministic layout rules for axis stacking/offset,
- interaction that targets the axis under the cursor (locks, wheel zoom),
- stable behavior across desktop and wasm.

`delinea` is headless: it owns data semantics (windows, transforms, hit testing), while the UI adapter
(`fret-chart`) owns pixel layout and rendering. Without an explicit contract, multi-axis behavior will
drift as new series types are added.

## Relationship to Other ADRs

- ADR 0190: headless engine boundary.
- ADR 0192: axis scales + coordinate mapping.
- ADR 0193: marks identity and renderer contract.
- ADR 0195: interaction and hit testing contract (axis locks / zoom locks).

## Decision

### 1) Axis placement is explicit and stable

Introduce an explicit `AxisPosition` for cartesian axes:

- X axes: `Top` / `Bottom`
- Y axes: `Left` / `Right`

Defaults:

- `AxisKind::X` defaults to `Bottom`.
- `AxisKind::Y` defaults to `Left`.

Invalid combinations are rejected by the model validator (e.g. `AxisKind::X` with `Left`).

### 2) `AxisPosition` is presentation-only (does not affect marks)

`AxisPosition` controls layout and drawing but does not change:

- data windows (`DataWindowX/Y`),
- transforms / row slicing,
- marks generation.

This avoids unnecessary recomputation when only axis placement changes.

### 3) Multi-axis layout in `fret-chart` (single grid, v1)

`fret-chart` v1 supports layout for a single active grid.

Grid selection:

- The active grid is derived from the primary series (first visible series in model order).
- Axes from other grids are ignored by the UI adapter (until multi-grid layout is designed).

Band layout:

- Each axis consumes one fixed band:
  - Y axis band width = `ChartStyle.axis_band_x`
  - X axis band height = `ChartStyle.axis_band_y`
- Multiple axes on the same side stack outward from the plot:
  - `Left` axes: first axis is closest to the plot; additional axes extend further left.
  - `Right` axes: first axis is closest to the plot; additional axes extend further right.
  - `Top` axes: first axis is closest to the plot; additional axes extend further up.
  - `Bottom` axes: first axis is closest to the plot; additional axes extend further down.

Order within a side:

- v1 uses `AxisId` ordering as a deterministic default.
- A future extension may add an explicit `offset`/`index` for axis ordering parity with ECharts.

### 4) Interaction targets the hovered axis region (recommended default)

The UI adapter derives an “axis region” from pointer position:

- If the pointer is inside an axis band rect, the hovered axis is that axis.
- Otherwise, the pointer is in the plot region.

Active axes in the plot region:

- `fret-chart` tracks an “active axis pair” (`active_x_axis`, `active_y_axis`).
- Hovering or interacting with an axis band updates the corresponding active axis.
- Plot-region interactions (pan/zoom/box zoom/reset) target the active axis pair.
- If no active axis has been selected yet, the active pair falls back to the primary axes
  (first visible series in model order).

Suggested shortcut policy (UI-level, but implemented via headless actions):

- `L`: toggle pan+zoom lock for the hovered axis (plot toggles primary X+Y).
- `Shift+L`: toggle pan lock.
- `Ctrl+L`: toggle zoom lock.

Wheel zoom targeting:

- In plot region: apply zoom to the active axis pair (subject to modifier policy).
- In axis band region: apply zoom to the hovered axis (and optionally the primary other axis when
  modifiers request “both”).

## Consequences

- Multi-axis charts become a straightforward, additive extension rather than a rewrite.
- Axis locks and zoom locks remain deterministic and unit-testable in `delinea`.
- Multi-grid layout remains a follow-up, but the contract avoids accidentally mixing axes across grids.

## Follow-ups

P0:

- Implement `AxisPosition` in `delinea::spec` and ensure it is validated in the model.
- Refactor `fret-chart` to draw multiple axes and to target the hovered axis for lock/zoom behaviors.

P1:

- Add explicit axis ordering (`offset` / `axis_index`) with documented composition rules.
- Multi-grid layout and per-grid interaction policies (axis pointer sync, per-grid zoom).

## References

- ECharts axis config concepts: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\coord\\cartesian\\AxisModel.ts`
- ADR 0190: `docs/adr/0190-delinea-headless-chart-engine.md`
- ADR 0195: `docs/adr/0195-delinea-interaction-and-hit-testing-contract.md`
