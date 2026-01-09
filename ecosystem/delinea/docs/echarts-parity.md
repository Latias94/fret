# ECharts Parity Checklist (delinea + fret-chart)

This document tracks our alignment with ECharts (Option model, cartesian charts, and interaction semantics).
It is intentionally scoped to the `delinea` headless engine and the `fret-chart` UI adapter.

## Scope

- In scope:
  - `delinea`: headless engine, data model, transforms, hit testing, LOD, interaction semantics.
  - `fret-chart`: renderer/UI adapter (layout, input, drawing, text), minimal styling.
- Out of scope:
  - `fret-plot` / `fret-plot3d`: ImPlot-like retained UI widgets (different design goals).
  - Non-cartesian coordinate systems (geo/radar/polar) unless explicitly added later.

## Status Legend

- ✅ Implemented
- 🟨 Partial / Minimal
- 🧪 Prototype
- ⏳ Planned
- ❌ Not planned (for now)

## Top-Level Architecture

| ECharts concept | Our equivalent | Status | Notes |
|---|---|---:|---|
| `option` | `ChartSpec` | 🟨 | We support a subset focused on cartesian 2D line-family charts. |
| `dataset` | `DatasetSpec` + `DatasetStore` | 🟨 | Field-to-column mapping is explicit via `FieldId` and `SeriesEncode`. |
| `encode` | `SeriesEncode` | ✅ | `x`, `y`, `y2` (for bands). |
| `series` | `SeriesSpec` (`SeriesKind`) | 🟨 | Currently `Line`, `Area`, `Band`, `Bar`, `Scatter`. |
| `grid` | `GridSpec` | 🟨 | Single-grid usage is primary; multi-grid is possible but not exercised. |
| `xAxis/yAxis` | `AxisSpec` + `AxisScale` + `AxisRange` | 🟨 | `Value/Category` scales; `Auto/LockMin/LockMax/Fixed` constraints. |
| `axisPointer` | `AxisPointerSpec` | 🟨 | `trigger=Item` and `trigger=Axis` supported (cartesian X). |
| `tooltip` | `AxisPointerOutput.tooltip` | 🟨 | Headless-generated structured lines; styling/layout in UI. |
| `legend` | `fret-chart` legend overlay | 🟨 | Basic list + visibility toggles; no paging/scrolling yet. |
| `dataZoom` | `DataZoomXSpec` + transform node | 🟨 | X-only for now; ECharts has richer modes/components. |
| `transform` | `transform` pipeline | 🟨 | Semantics + ordering documented; limited nodes implemented. |

## Series Types

| ECharts series | Status | Our plan / mapping |
|---|---:|---|
| `line` | ✅ | `SeriesKind::Line` |
| `area` (line with areaStyle) | ✅ | `SeriesKind::Area` + `AreaBaseline` |
| `band` (custom, common in finance) | ✅ | `SeriesKind::Band` (`encode.y` + `encode.y2`) |
| `line` + `stack` | 🟨 | `SeriesSpec.stack` + `stack_strategy` implemented for `Line` (v1: no stack transforms yet; no stacked areas/bars). |
| `scatter` | 🟨 | `SeriesKind::Scatter` + point marks + hit test + pixel-bounded large mode; missing symbol/size options. |
| `bar` | ✅ | `SeriesKind::Bar` (requires Category X axis in v1). |
| `candlestick` | ❌ | Needs OHLC encode + mark layout. |
| `heatmap` | ❌ | Needs 2D binning + texture/mesh strategy. |
| `boxplot` | ❌ | Needs statistical transforms + marks. |
| `graph` | ❌ | Needs force/layout + edges. |
| `pie` | ❌ | Non-cartesian. |
| `radar/polar` | ❌ | Non-cartesian. |
| `surface` / 3D | ❌ | Requires separate coordinate + renderer path. |

## Components & Interaction Semantics

### Tooltip / AxisPointer

| Feature | Status | Notes |
|---|---:|---|
| `tooltip.trigger=item` | ✅ | Implemented as `AxisPointerTrigger::Item`. |
| `tooltip.trigger=axis` | 🧪 | Implemented for shared X-axis; per-series values via interpolation; assumes monotonic X. |
| Crosshair rendering | ✅ | `fret-chart` draws crosshair lines. |
| Marker dot | 🟨 | Drawn when a hit is available (`AxisPointerOutput.hit`). |
| Formatter functions | ❌ | No user-provided formatting callbacks yet (planned as a headless formatter API). |

### DataZoom

| Feature | Status | Notes |
|---|---:|---|
| `dataZoom` filtering | 🟨 | X-only. `filterMode` aligns with ECharts core semantics. |
| `filterMode=filter` | ✅ | Filters points outside the X window (affects bounds/LOD). |
| `filterMode=none` | ✅ | Keeps Y bounds global while X window changes. |
| `inside` (wheel/drag) | 🧪 | UI supports pan/zoom gestures; not yet modeled as separate component types. |
| `slider` UI | ❌ | Not implemented. |
| Y / 2D zoom | ❌ | Not implemented. |

### Legend

| Feature | Status | Notes |
|---|---:|---|
| `legend` list | 🟨 | In `fret-chart`, top-right overlay. |
| Toggle series visibility | ✅ | Click toggles `Action::SetSeriesVisible`. |
| Hover highlight | 🟨 | Implemented as UI-only fade; no headless highlight model yet. |
| Multiple legend types | ❌ | Scroll/paging/selection groups not yet. |

## Data & Performance

| Topic | Status | Notes |
|---|---:|---|
| Large dataset handling | 🟨 | Marks stage uses LOD (min/max-per-pixel) for polyline downsampling. |
| Incremental work scheduling | 🟨 | Engine uses `WorkBudget` and can finish across multiple `step()` calls. |
| Zero-copy column access | 🟨 | `DataTable` stores columns; series refer via column indices through `DatasetSpec.fields`. |
| Columnar transforms | ⏳ | Planned: more transform nodes (filter/aggregate/sort) and cached indices. |
| Monotonic X assumption | 🧪 | `trigger=Axis` currently uses binary search + linear interpolation; falls back to missing value on invalid ranges. |

## Known Gaps (High Priority Candidates)

1. Multi-axis / multiple y-axes (layout + mapping + label policy).
2. Stacking transforms (stacked bars/areas) and derived columns.
3. Scatter parity: symbol/size options, large/progressive thresholds, and richer hit testing policies.
4. Time/log scales + tick formatting policies.
5. Styling/theming surface (colors, line styles, per-series overrides, legends/tooltip formatters).
6. `dataZoom` slider UI + Y-axis zoom (and 2D brush/selection).

## References

- ADR 0128: `docs/adr/0128-delinea-headless-chart-engine.md`
- ADR 0129: `docs/adr/0129-delinea-transform-pipeline-and-datazoom-semantics.md`
- ADR 0130: `docs/adr/0130-delinea-axis-scales-and-coordinate-mapping.md`
- ADR 0131: `docs/adr/0131-delinea-marks-identity-and-renderer-contract.md`
- ADR 0132: `docs/adr/0132-delinea-large-data-and-progressive-rendering.md`
- ADR 0133: `docs/adr/0133-delinea-interaction-and-hit-testing-contract.md`
- ADR 0134: `docs/adr/0134-delinea-multi-axis-and-layout-contract.md`
- `ecosystem/delinea/docs/transform-pipeline.md`
