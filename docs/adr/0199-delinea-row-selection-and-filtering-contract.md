# ADR 0199: `delinea` RowSelection + Filtering Contract (ECharts-Inspired)

Status: Accepted (P0)

## Context

`delinea` is intended to support “application charts” with an ECharts-inspired mental model
(dataset + encode + transform pipeline + components like dataZoom) while remaining:

- renderer-agnostic,
- deterministic and unit-testable,
- performant on large datasets (bounded work, minimal allocations).

Today, our pipeline already relies on a selection abstraction:

- `RowRange` (continuous slice of a dataset table),
- `RowSelection` (`All`, `Range(RowRange)`, or `Indices(...)`),
- X windowing via `DataZoomXSpec` + `FilterMode::{Filter,None}` (ADR 0191),
- Y and 2D view windows (mapping-only; ADR 0198).

This is sufficient for X-only slicing on monotonic columns, but we also need an explicit path for:

- ECharts `filterMode` variants (`weakFilter`, `empty`) and multi-dimensional filtering,
- 2D brush/selection semantics beyond “write view windows”,
- future dataset transforms (stacking/aggregation) that may need derived columns and/or sparse row selection,
- consistent composition rules that avoid later rewrites.

ECharts filters series data using mechanisms like `selectRange` and, for some modes, per-row decisions.
Those semantics usually imply **sparse** selection or value masking, which can be expensive if introduced
without an explicit contract and budget-aware execution.

This ADR locks the minimal selection/filter contract that keeps v1 fast while leaving an explicit path
to ECharts-class behaviors.

## Relationship to Other ADRs

- ADR 0190: headless engine boundary.
- ADR 0191: transform pipeline + X `FilterMode` semantics.
- ADR 0194: large data + progressive rendering strategy.
- ADR 0195: interaction + hit testing contract.
- ADR 0198: Y + 2D zoom semantics (mapping-only Y in v1).

## Decision

### 1) Selection is a first-class transform output

`RowSelection` is the canonical “which rows participate” contract between:

- the transform pipeline (selection derivation),
- the view state (per-series inputs),
- marks/LOD/bounds/hit testing (consumers).

Selection is always derived from a **base dataset row range**:

- Base range comes from `ChartState.dataset_row_ranges[dataset]` (optional) and clamps to dataset length.
- All selection operations must be expressible as `selection = f(base_range, transforms, view_state)`.

### 2) v1 prefers contiguous selection (fast path)

P0/v1 preference:

- Selection will be either `All` or `Range(RowRange)`.
- No sparse selection is required to implement:
  - X window slicing (`FilterMode::Filter`),
  - Y zoom/pan,
  - 2D box zoom (as paired view windows).

This preserves:

- O(log n) slicing on monotonic X,
- stable caching keys,
- low memory overhead.

### 3) v1 also allows indices selection as an internal transform cache

To align with ECharts `DataStore` (`_indices` + `getRawIndex`) and to keep future filtering extensible,
`RowSelection` supports a non-contiguous form:

- `RowSelection::Indices(Arc<[u32]>)`

v1 constraints:

- Indices are **raw row indices** (view index -> raw index mapping).
- Indices may be produced by budgeted engine stages (ADR 0190) and cached by revision keys.
- Consumers (marks/LOD/bounds/hit testing) must iterate selection via `get_raw_index(...)` rather than
  assuming contiguity.

This does not remove the contiguous fast path. It makes “sparse views” an explicit, testable contract
instead of an ad-hoc future rewrite.

### 4) Selection does not *encode* “empty” masking (selection vs masking stay separate)

ECharts `filterMode='empty'` keeps rows but turns out-of-window values into `NaN`, causing line breaks.

We keep **two separate concepts**:

- **row participation** (selection), vs
- **value validity/masking** (per-point visibility and segment-break rules).

In v1, we implement an ECharts-aligned subset without conflating the two:

- `FilterMode::Filter` removes out-of-window rows by selection (contiguous slice when possible).
- `FilterMode::None` keeps the base range unchanged (no out-of-window masking).
- `FilterMode::Empty` preserves the base row selection but treats out-of-window samples as missing at mark emission
  time (line-family breaks), following ADR 0211 + ADR 0203.

### 5) When sparse behaviors are added, they must be budget-aware and cacheable

When we introduce ECharts-like `weakFilter` / `empty` / multi-dimensional filtering, we also introduce
new internal representations. The contract constraints:

- Selection and/or masking must be computed under `WorkBudget` and be resumable across `step()` calls.
- Selection/masking results must be cacheable by:
  - dataset revision,
  - model revision (encode/series config),
  - transform parameters (windows/modes),
  - view revision (state changes).
- The engine must remain deterministic given the same inputs and budget progression.

### 6) P1 extension path: add value masking as a separate concept

We explicitly separate two future mechanisms:

1. **Value masking** (rows kept, but values become invalid):
   - Add a per-series “validity” channel or derived-column transform that can mark points as `NaN`.
   - Used for ECharts `empty` semantics (line breaks without removing rows).

This avoids conflating “filtering” with “rendering rules” and keeps marks/hit-testing rules explicit.

### 7) Composition order is fixed and documented

To avoid future drift, we adopt a stable transform ordering for cartesian grids:

1. Base dataset row range clamp.
2. X selection transforms (including DataZoomX `FilterMode`).
3. Y selection transforms (P1; if introduced).
4. Derived-column transforms (stacking/aggregates) that are defined over the effective selection.
5. LOD transforms (min/max per pixel, sampling) over the selected (and/or masked) view.

This is consistent with the forward-compatibility rule in ADR 0191 (“apply X filters before Y filters”).

### 8) “Decide early” ECharts-inspired capabilities (backlog)

The following ECharts concepts are likely to force refactors if added late. They are not all P0 features,
but the *contracts* should be decided early:

DataZoom:

- `rangeMode` (`percent` vs `value`): decide whether `DataWindow` is always “data value space” (recommended)
  and whether percent windows are UI-only.
- Span constraints: `minSpan/maxSpan` and `minValueSpan/maxValueSpan` (and how they compose with `AxisRange`).
- `zoomLock`: treat as interaction gating (like locks), not as a hard override of programmatic window writes.
- Multiple dataZoom components per axis (inside + slider): decide explicit composition rules (intersection vs “hosted by”).
- Action coalescing / throttling: define where throttling lives (UI vs headless) and how it affects determinism.

Transforms:

- Derived columns: decide how to represent and cache computed columns (stack base/top, aggregates).
- Category/time/log transforms: decide how scale parsing and tick formatting interact with selection and windows.

Interaction and output:

- Brush selection: decide whether brush produces a durable selection output (indices/mask) or only writes view windows.
  - v1 baseline: brush selection is a headless output (does not write view windows); see ADR 0205.
- Tooltip formatting contract: decide headless formatter surfaces (typed values + series meta) vs UI-only formatting.
- Visual mapping: decide whether “visualMap”-like encodings (color/size/opacity) are a first-class contract or UI-only.

Large data and streaming:

- Progressive thresholds: ECharts-style `progressive` / `progressiveThreshold` and `large` / `largeThreshold`
  (and how they map onto ADR 0194 `WorkBudget` + LOD stages).
- Data updates: `setOption` merge semantics and `appendData`-like streaming (whether we support incremental
  dataset appends without re-tessellating everything, and what invalidates cached selections/LOD).
  This should remain a headless contract expressed via revisions and budgeted work, not UI-only behavior.

## Consequences

- v1 keeps the engine fast and simple: contiguous row slicing and mapping-only Y/2D zoom.
- We explicitly reserve space for ECharts-class filtering without forcing a redesign of the v1 engine.
- Future work can introduce sparse selection and masking in a budget-aware, cacheable manner.

## Follow-ups

P0:

- Keep v1 behaviors aligned with ADR 0191 and ADR 0198 (contiguous fast path remains primary).
- Add conformance demos that stress multi-axis + 2D box zoom + large datasets.

P1:

- Add `FilterMode::{WeakFilter,Empty}` only after:
  - stacking/bar/categorical behaviors are locked,
  - masking semantics are defined and tested.

## Amendments

- 2026-01-13 (ECharts replica workstream): `DataZoomYSpec.filter_mode` can opt-in to materializing
  sparse `RowSelection::Indices` for Y filtering on non-stacked scatter and line-family series (Line/Area).
  Consumers that implement “empty-style” segment breaks (e.g. `FilterMode::Empty` line-family marks)
  must treat selection as authoritative and iterate via `get_raw_index(...)` rather than expanding
  to a contiguous bounding `RowRange` via `as_range(...)`.
