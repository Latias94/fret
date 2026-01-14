# Apache ECharts Alignment (Fret Charts)

This document tracks feature and semantics alignment between:

- Apache ECharts (reference in `repo-ref/echarts`)
- `delinea` (headless chart engine)
- `fret-chart` (UI adapter: layout, input routing, overlays, and token-driven styling)

It complements `docs/delinea-echarts-alignment.md`, which is the detailed cartesian conformance
checklist used to lock down refactor-safe semantics.

Status symbols:

- `[x]` implemented
- `[~]` partial / intentionally different (v1 boundary)
- `[ ]` planned / missing

## Scope

- Primary focus: ECharts-class 2D cartesian charts (dataset/encode, dataZoom, axisPointer/tooltip, legend, brush).
- Non-goals: option-schema parity, HTML/rich text rendering parity, and the full ECharts plugin ecosystem.
- 3D: out of scope here (ECharts 3D lives in `echarts-gl`); track Plot3D parity separately.

## Architecture Mapping (ECharts -> Fret)

- `ecModel`/`SeriesModel`/`ComponentModel` -> `delinea::ChartModel` (validated spec graph)
- `AxisProxy` + `dataZoomProcessor` -> `delinea` staged pipeline (future: a dedicated filter processor stage)
- `DataStore` raw index identity -> `RowSelection` + `get_raw_index`
- `zrender` display list -> `delinea::MarksOutput` -> `fret-chart` scene ops (`SceneOp::{Path,Quad,Text}`)

## Parity Map (High Level)

### Data model & transform pipeline

- `[x]` Dataset/field indirection + `encode` mapping (ADR 1128 / ADR 1140)
- `[x]` Stable raw-index identity across transforms (`RowSelection`) (ADR 1137 / ADR 1140)
- `[~]` DataZoom filter modes (`Filter`/`None`/`WeakFilter`/`Empty`) with a v1 multi-dim subset (ADR 1129 / ADR 1150)
- `[~]` Order-sensitive multi-dim filtering (ECharts “filter X then reset/filter Y”) (planned; needs processor stage)
- `[ ]` General transform graph with cached node outputs + derived columns (ECharts-class dataset transforms)
- `[ ]` Dataset transform operators (filter/map/sort/aggregate) as first-class nodes (beyond dataZoom)

### Coordinate systems & layout

- `[x]` Cartesian grid with multi-axis routing (ADR 1134)
- `[ ]` Multi-grid layout (multiple independent grids in one chart)
- `[ ]` Polar coordinate system
- `[ ]` Geo / map coordinate system
- `[ ]` Calendar coordinate system
- `[ ]` Single-axis coordinate system (and associated components)

### Series types (ECharts-class)

- `[x]` Line / Area / Band / Bar / Scatter (cartesian)
- `[~]` Candlestick / OHLC (available in `fret-plot` demos, but not yet in `delinea`/`fret-chart`)
- `[ ]` Heatmap (headless chart stack; `fret-plot` has a separate heatmap implementation)
- `[ ]` Pie / Radar / Gauge / Funnel
- `[ ]` Treemap / Sunburst
- `[ ]` Graph / Sankey

### Components & interaction semantics

- `[x]` dataZoom X inside + slider UI (`fret-chart`) (ADR 1129 / ADR 1138)
- `[~]` dataZoom Y + 2D zoom semantics (v1 boundary + opt-in filtering) (ADR 1136 / ADR 1150)
- `[x]` AxisPointer (axis-trigger + item-trigger) baseline (ADR 1133)
- `[~]` Tooltip formatting contract (structured rows + hooks; missing rich text/HTML parity) (ADR 1148)
  - Item-trigger defaults are ECharts-aligned (`TooltipSpecV1.item_axis_line=hide` by default; axis values are shown via axisPointer labels when enabled).
- `[x]` Legend visibility and isolation semantics (`Action::SetSeriesVisible`) (ADR 1128; UX tracked by `docs/delinea-echarts-alignment.md`)
- `[~]` Brush selection output + link events (ADR 1144 / ADR 1146; parity tests still sparse)
- `[~]` VisualMap baseline (continuous + piecewise) (ADR 1147; channel coverage is incomplete)
- `[ ]` Toolbox / title / timeline components

### Styling & state model

- `[~]` Token-driven chart styling (tracked in ADR 0142; UI adapter work)
- `[ ]` ECharts-style emphasis / blur / downplay state model (including interaction-driven highlight policies)
- `[ ]` Universal transitions and animation parity (series transitions, progressive animation)
- `[ ]` Label layout and collision avoidance (including rich text)

### Performance & large data

- `[x]` Budgeted progressive stepping (`WorkBudget`) (ADR 1132)
- `[~]` Large-data knobs parity (`large`, `progressive`, sampling indices) (subset implemented; more series coverage needed)
- `[ ]` Incremental dataset updates and stable partial recompute (processor-level caches keyed by revision)

## Recommended Next Steps (ECharts Replica Workstream)

1. P0: Add a dedicated “filter processor” stage (ECharts `dataZoomProcessor` analogue) that owns ordering-sensitive
   composition (X-before-Y) and outputs a unified per-series participation contract (selection + masks).
2. P0: Add a general transform graph with cached node outputs + derived columns (ECharts-class dataset transforms).
3. P0: Multi-grid layout (engine layout + UI adapter routing) and a conformance harness that locks routing invariants.
4. P1: Extend `Empty` parity beyond line-family (scatter/bar mark emission + tests), keeping tooltip/axisPointer/hit-test
   consistent via the shared mask contract.
5. P1: Expand conformance coverage in `apps/fret-examples/src/chart_multi_axis_demo.rs` and lock regression tests for:
   - 2D dataZoom ordering rules,
   - visualMap + tooltip/axisPointer interactions.

## Evidence Anchors

- Core conformance checklist: `docs/delinea-echarts-alignment.md`
- ADR alignment table: `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- Key implementation surfaces:
  - Engine stages: `ecosystem/delinea/src/engine/stages/`
  - View participation: `ecosystem/delinea/src/view/mod.rs`
  - UI adapter: `ecosystem/fret-chart/src/retained/canvas.rs`
