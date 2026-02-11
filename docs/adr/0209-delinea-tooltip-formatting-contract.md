# ADR 0209: `delinea` Tooltip Formatting Contract (Structured Payload + UI Formatter v1)

Status: Proposed

## Context

We want ECharts-class tooltip behavior (`trigger=item` / `trigger=axis`) while keeping:

- deterministic ordering (stable under refactors),
- a headless engine surface that is renderer/UI agnostic,
- a UI adapter surface that can apply theme + application-specific formatting,
- a clear contract for missing values and snapping behavior.

The existing v1 implementation builds pre-formatted `label/value` strings inside the engine.
That makes it hard to:

- provide formatter hooks (ECharts `tooltip.formatter`-style),
- keep formatting concerns out of the headless layer,
- evolve tooltip layout (multi-column, rich text, units) without rewriting engine code.

## Relationship to Other ADRs

- ADR 0190: `delinea` headless engine boundary.
- ADR 0192: axis mapping and scale contracts (required for axis-trigger tooltip).
- ADR 0195: interaction + hit testing contract (axisPointer/tooltip baseline semantics).
- ADR 0131: token-driven chart styling (UI-side theme integration for chart components).

## Decision

### 1) The headless engine emits a structured tooltip payload (no pre-formatted strings)

`delinea` emits `TooltipOutput` as part of `AxisPointerOutput`:

- For `trigger=Item`: a single-series payload based on a hit result.
- For `trigger=Axis`: an axis-trigger payload that contains:
  - the trigger axis identity and axis value in data space,
  - one entry per visible series in `series_order`,
  - `Missing` values for unsampleable series (NaN / out-of-range / no data).

This keeps tooltip semantics and sampling rules in the engine, while moving presentation
to the adapter layer.

### 2) Formatting happens in the UI adapter (`fret-chart`) via a formatter hook

`fret-chart` owns the tooltip "view model" and default formatting:

- It maps `AxisId`/`SeriesId` to display labels (`name` or fallbacks).
- It formats numeric values using the axis scale + current window.
- It renders tooltip panels and text blobs.

Option-level formatting configuration lives in the chart spec as `ChartSpec.tooltip: Option<TooltipSpecV1>`.
The adapter applies `TooltipSpecV1` templates/decimals by default. Applications may override the
entire formatting pipeline by calling `ChartCanvas::set_tooltip_formatter(...)`.

`TooltipSpecV1` supports per-series overrides via `TooltipSpecV1.series_overrides`.

The adapter exposes an optional formatter hook so apps can implement:

- unit suffixes, fixed decimals, scientific notation,
- per-series formatting rules,
- richer layouts (multi-column, custom ordering) without changing the engine.

### 3) Conformance invariants (P0)

These invariants are treated as stable contracts for refactors:

- Ordering:
  - Axis-trigger: the first row is the trigger axis value, then series rows in `series_order`.
  - Item-trigger: the first row is the series identity, then axis/value rows.
- Missing values: represented explicitly as `Missing` (rendered as `-` by the default formatter).
- Snapping:
  - When `AxisPointerSpec.snap=true` and a close-enough hit exists, the crosshair may snap to the
    hit point.
  - Axis-trigger tooltips use the snapped axis value when snapping is active.
- No panics: tooltip generation must never panic on NaN/missing data.

## Implementation Notes

- Engine types: `ecosystem/delinea/src/tooltip.rs`
- Engine wiring: `ecosystem/delinea/src/engine/mod.rs` (`compute_*_axis_pointer_output`)
- Performance notes:
  - Axis-trigger sampling must remain budget-aware and avoid O(n) scans for very large non-monotonic views.
  - v1 uses a budgeted nearest-X index (`NearestXIndexStage`) to recover near-O(log n) sampling for axis-trigger
    tooltips and snapping without requiring monotonic X inputs. The stage supports append-only resume and prefix
    reuse when the request end grows.
- UI formatting + rendering: `ecosystem/fret-chart/src/retained/tooltip.rs`, `ecosystem/fret-chart/src/retained/canvas.rs`
- Regression tests:
  - `ecosystem/delinea/src/engine/tests.rs` (axis-trigger multi-series ordering + missing values)
  - `ecosystem/fret-chart/src/retained/tooltip.rs` (template + decimals formatting)
  - `apps/fret-examples/src/chart_demo.rs` (manual validation)

## Validation

- Desktop: `cargo run -p fret-demo --bin fret-demo -- chart_demo`
- Web: `cargo run -p fretboard -- dev web --demo chart_demo`
