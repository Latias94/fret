# ADR 0200: `delinea` DataZoom Component Composition + Span Policy (ECharts-Inspired)

Status: Accepted (P0)

## Context

We already have a working v1 zoom/pan surface:

- Durable X dataZoom defaults (`DataZoomXSpec.filter_mode`) and X window state (`ChartState.data_zoom_x[axis]`).
- Y view windows (`ChartState.data_window_y[axis]`) and 2D view updates (`Action::SetViewWindow2D`) (ADR 0198).
- Interaction-derived 2D box zoom uses `Action::SetViewWindow2DFromZoom` so span limits can be applied
  without changing programmatic writes (ADR 0198).
- Interaction gating via locks (`AxisInteractionLocks`) (ADR 0197).
- A transform/view pipeline that currently supports contiguous selection and X slicing (ADR 0191, ADR 0199).

The next “hard-to-change” decisions that will impact slider UI, brush zoom, and multi-axis charts are:

- Whether dataZoom windows are stored in **value** space or **percent** space.
- How to express and apply **span constraints** (min/max span) and whether they gate programmatic writes.
- How to handle **multiple dataZoom components per axis** (ECharts: slider + inside share an `AxisProxy`).
- Where **throttling/coalescing** lives without breaking determinism or large-data budgets.

Apache ECharts provides useful precedent:

- `rangeMode` (`value` vs `percent`) affects how a dataZoom window behaves when the data extent changes.
- `minSpan/maxSpan` and `minValueSpan/maxValueSpan` constrain interactive zoom ranges.
  - In `delinea` v1, we expose this as value-space limits on `DataZoomXSpec`:
    - `min_value_span: Option<f64>`
    - `max_value_span: Option<f64>`
- Multiple dataZoom models can target the same axis and share an axis proxy; the proxy applies composition.
- `zoomLock` is treated as an interaction constraint rather than preventing API-driven range changes.

This ADR locks down a Fret-native subset that stays compatible with our headless boundary and performance goals.

## Relationship to Other ADRs

- ADR 0191: dataZoom/filter semantics and transform pipeline shape.
- ADR 0194: large data and progressive work budgets.
- ADR 0196: multi-axis targeting rules in `fret-chart`.
- ADR 0197: axis locks as interaction gating.
- ADR 0198: Y + 2D zoom semantics.
- ADR 0199: row selection vs masking split (future `weakFilter/empty`).

## Decision

### 1) `DataWindow` is always stored in data value space

All effective zoom windows in `delinea` are expressed as:

- `DataWindow { min, max }` in **data value space** for the corresponding axis.

This applies to:

- X windows (`ChartState.data_zoom_x[axis].window`),
- Y windows (`ChartState.data_window_y[axis]`),
- 2D writes (`Action::SetViewWindow2D`).

Percent space is treated as a **UI representation**, not a headless state representation:

- slider handles, scrollbars, and minimaps may compute percent windows from a representative data extent.
- those percent windows must be converted into value-space windows before dispatching headless actions.

Rationale:

- value-space windows are stable across axis scales (value/time/log) and avoid precision ambiguity,
- headless transforms and caches are keyed by deterministic values,
- the engine remains agnostic to UI layout and “100%” definitions.

### 2) v1 supports at most one dataZoom component per axis

We keep the v1 restriction introduced in ADR 0191:

- In `ChartSpec`, at most one `DataZoomXSpec` may reference a given X axis.

This avoids prematurely designing composition semantics and keeps model validation simple.

Extension path (P1):

- Allow multiple `DataZoom*Spec` per axis and define an explicit composition rule (see §4).

### 3) Span constraints are an interaction policy, not an API gate

We adopt the same core philosophy as ECharts `zoomLock`:

- span constraints are intended to constrain **interactive** zoom operations,
- they must not prevent programmatic initialization or explicit API writes.

We define two categories of window updates:

**Interaction-derived updates** (must respect span constraints):

- `Action::PanDataWindow*FromBase`
- `Action::ZoomDataWindow*FromBase`
- `Action::SetDataWindow*FromZoom` (slider / 1D zoom writes)
- `Action::SetViewWindow2DFromZoom` (2D box zoom)

**Programmatic updates** (do not apply span constraints automatically):

- `Action::SetDataWindowX`
- `Action::SetDataWindowY`
- `Action::SetViewWindow2D`

In both cases, `AxisRange` constraints (Fixed/LockMin/LockMax) still apply (ADR 0198).

Rationale:

- callers must be able to set initial ranges even if interactive zoom is “locked” to a min span,
- it keeps the headless API usable for apps that implement their own UI sliders or animations.

### 4) Future multi-component composition rule: intersection in value space

When we allow multiple zoom components per axis (P1), we will adopt a deterministic composition rule:

- compute each component’s effective value-space window,
- compose them by **intersection** (tightest window wins),
- apply `AxisRange` constraints after composition.

Notes:

- if the intersection is empty/degenerate, clamp to a non-degenerate span using the span policy (min span),
  or fall back to the representative axis window for that axis.
- composition must remain stable under partial state (e.g. slider not yet initialized).

This is equivalent in spirit to ECharts’ shared `AxisProxy` hosting multiple dataZoom models, but expressed in a
headless, deterministic manner.

### 5) `rangeMode` is a durable spec concern (P1), not a view-state concern (P0)

ECharts `rangeMode` influences how a zoom window is maintained across data extent changes.

In `delinea`:

- P0: we do not expose `rangeMode` publicly; windows are value-space in view state, and any “percent anchoring”
  behavior is handled by the UI (or the caller) by recomputing value windows as needed.
- P1: we may add a durable range mode to `DataZoom*Spec` to support ECharts-like behavior when:
  - data is appended/streamed,
  - axes use category/time scales where “extent” may change under merges.

If introduced, `rangeMode` must be defined as:

- a durable policy that affects how a component **derives value-space windows** from percent-based UI inputs
  when the representative extent changes.

It must not change the internal representation: the engine still stores effective windows in value space.

### 6) Throttling and action coalescing live in the UI adapter layer

To preserve determinism and keep headless scheduling simple:

- `delinea` does not implement a time-based throttle for dataZoom actions.
- UI adapters (e.g. `fret-chart`) are responsible for:
  - reducing redundant actions (coalescing rapid pointer deltas),
  - enforcing a maximum update rate for slider drags if needed,
  - respecting `WorkBudget`/progressive rendering by avoiding “action storms”.

Headless side expectations:

- given the same sequence of actions, the engine produces the same results.
- performance is governed by `WorkBudget` and incremental `step()` scheduling (ADR 0194), not by time-based drops.

## Consequences

- Slider and inside zoom can share the same headless engine without a new “percent window” state surface.
- We can add span constraints without breaking programmatic control.
- Multi-component composition is reserved with a clear deterministic rule (intersection), aligned with ECharts’ proxy idea.
- UI adapters remain responsible for interactive feel (throttle/realtime) while headless stays deterministic.

## Follow-ups

P0:

- Keep `DataWindow` value-space only and ensure all UI actions produce value-space windows.
- When adding slider UI, define the representative extent used to render percent handles (UI-only).

P1:

- Add multiple dataZoom components per axis with intersection composition.
- Introduce `rangeMode` in spec once streaming/append data is designed (see ADR 0199 backlog).
