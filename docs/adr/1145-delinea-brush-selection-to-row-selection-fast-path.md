# ADR 1145: `delinea` Brush Selection -> Row Selection Fast Path (P0)

Status: Accepted (P0)

## Context

ECharts-style brush selection ultimately selects *data items* (indices), but `delinea` v1 prioritizes:

- large-data responsiveness,
- predictable memory usage (avoid per-drag allocations),
- deterministic stepping under `WorkBudget`.

ADR 1144 defines brush selection as a headless **selection output** (not a view-window write), exposed as a
data-space 2D rectangle (`BrushSelection2D`).

However, many downstream use-cases (linking, external selection consumers, basic highlight policies) need a
row-level selection view. We want a minimal v1 mapping that is:

- allocation-free in the common case,
- `O(log n)` where possible,
- explicitly constrained (so we can extend without rewriting).

## Relationship to Other ADRs

- ADR 1137: `RowSelection` and filtering contract (contiguous fast path).
- ADR 1140: dataset storage and index identity.
- ADR 1144: brush selection output contract.

## Decision

### 1) v1 exports a derived contiguous row range per series (X-only)

When a cartesian brush selection is active, `delinea` derives a per-series contiguous row range based on the
brush X window:

- `ChartOutput.brush_x_row_ranges_by_series: BTreeMap<SeriesId, RowRange>`

This derived range is:

- scoped to series whose axis pair matches the brush selection (`series.x_axis == x_axis` and `series.y_axis == y_axis`),
- computed from the *effective* series view selection (after base row range and X dataZoom filtering),
- derived using the monotonic-X fast path (`row_range_for_x_window`) when the current series selection is contiguous.

Opt-in extension (multi-grid linking):

- When explicitly enabled via `LinkConfig.brush_x_export_policy`, the engine may additionally derive X row ranges
  for other visible series that share the same `(dataset, encode.x)` as at least one brushed series. This allows
  cross-grid "X linking" without changing the authoritative brush selection shape (still 2D, still scoped to a grid).

### 2) Y is not applied in v1 (no sparse selection yet)

The derived output is intentionally **X-only** in v1:

- Applying a 2D rectangle to points generally requires sparse indices (`RowSelection::Indices`) or value masking.
- Those features are deferred and must be introduced as explicit, budget-aware contracts (ADR 1137 follow-ups).

The authoritative selection rectangle remains available via `ChartOutput.brush_selection_2d` (ADR 1144), so a UI
adapter may still render a 2D brush overlay.

### 3) Brush actions do not bump the view revision

Brush selection updates do not bump `ChartState.revision`. The derived row ranges are computed during `step()` and
exposed via `ChartOutput`. This avoids forcing `ViewState` rebuilds for every pointer move while still allowing
consumers to observe the current selection.

## Consequences

- We can support “commercial-grade” interactive brushing with no per-drag allocations in v1.
- We gain a minimal selection output usable for linking and basic highlight policies.
- Full ECharts-class 2D item selection remains a P1+ extension (indices/masking).
