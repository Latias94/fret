# ADR 0201: `delinea` Time Axis Contract (Ticks + Default Labels)

Status: Proposed

## Context

`delinea` is an ECharts-inspired, headless chart engine (ADR 0190). Our v1 scale contract started
with `Value` and `Category` axes (ADR 0192). To align with real-world charting needs (timeseries,
finance, dashboards), we need a first-class **time axis** with deterministic tick generation and
reasonable default labels.

ECharts defines `type: 'time'` using a `TimeScale` implementation that:

- represents values as **milliseconds since epoch** (JS `Date`-compatible),
- selects an interval based on an approximate tick spacing,
- generates ticks aligned to time-unit boundaries (calendar-aware),
- formats labels with a level-aware formatter (locale + `useUTC`).

We want the same mental model and the same “hard-to-change” invariants, but we are explicitly not
trying to replicate ECharts’ full locale/time-zone formatting stack in v1.

## Relationship to Other ADRs

- ADR 0190: `delinea` headless chart engine.
- ADR 0192: axis scales + mapping contract (now extended by this ADR).
- ADR 0194: large-data + progressive work scheduling (tick generation must be bounded).

## Decision

### 1) Add an explicit `Time` axis scale kind

Extend `AxisScale` with:

- `AxisScale::Time(TimeAxisScale)`

This keeps “axis kind” explicit and prevents implicit, data-driven axis-type drift.

### 2) Time values are epoch milliseconds (`f64`)

For `AxisScale::Time`:

- axis values are **milliseconds since Unix epoch** (1970-01-01T00:00:00Z),
- stored as `f64` in datasets and windows,
- tick values are generated on millisecond boundaries (rounded to `i64` milliseconds).

This matches the ECharts default representation (`Date.getTime()`), and keeps time axes compatible
with our existing numeric pipeline (windows, transforms, dataZoom) without inventing a parallel
date-time value type.

### 3) Tick generation is deterministic and bounded

Given an axis `window = [min, max]` and `target_count`:

- The tick list is **monotonic**.
- The output **includes the endpoints** (`min` and `max`) to match our `nice_ticks` behavior and
  ECharts’ `TimeScale.getTicks` (which always adds extent endpoints).
- We generate “internal” ticks aligned to unit boundaries, based on a selected `(unit, step)` pair.
- The generation is **bounded** by a hard cap (10k steps) to avoid pathological loops.

### 4) Unit + step selection follows the ECharts shape

We select a tick “unit” and an integer step based on `approx_interval_ms = span / target_count`,
similar to ECharts’ `calcNiceTicks`:

- sub-second: `Millisecond` with `1/2/5/...` steps,
- seconds/minutes: `1/2/5/10/15/20/30`,
- hours: `1/2/3/4/6/12`,
- days: `1/2/3/5/7/14`,
- months: `1/2/3/6`,
- years: `1/2/5/10/...` (nice step on the year magnitude).

This preserves the important property: zooming/panning changes the chosen tick unit gradually
rather than causing unstable tick density.

### 5) Default label formatting is UTC-only in v1

v1 default formatting:

- uses **UTC** (no local time-zone handling yet),
- chooses a compact format based on the selected tick unit and the visible span:
  - large spans show dates (`YYYY-MM-DD`, `YYYY-MM`, `YYYY`),
  - small spans show time-of-day (`HH:mm`, `HH:mm:ss`, `HH:mm:ss.SSS`),
  - when the span is ≥ 1 day, we include the date prefix for sub-day ticks.

This is intentionally a minimal, stable default suitable for demos and internal tools.

Non-goals for this ADR:

- locale-aware formatting,
- time-zone selection (`useUTC=false`),
- custom label formatter callbacks.

Those will be addressed in a follow-up ADR once the component/theming surface is clearer.

## Consequences

### Benefits

- Enables real timeseries charts without forcing users to treat time as a plain numeric axis.
- Keeps time axes compatible with the rest of the engine (windows, transforms, dataZoom).
- Provides deterministic, bounded tick generation that works with progressive rendering budgets.

### Trade-offs

- UTC-only default labels may differ from ECharts’ “local by default” presentation.
- The default formatter is intentionally conservative; higher-quality formatting is deferred.

## Follow-ups

1. Add `Log` scale (`type: 'log'`) with correct mapping semantics and tick/label policy.
2. Add locale + time-zone policies (ECharts `useUTC`-like behavior).
3. Add user-provided formatter hooks (headless contract; UI just renders strings).
