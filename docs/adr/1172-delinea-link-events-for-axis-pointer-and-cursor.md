# ADR 1172: `delinea` Link Events for AxisPointer/Cursor (P0)

Status: Proposed (P0)

## Context

`delinea` already supports:

- a portable, headless interaction model (`Action` + `ChartState`),
- deterministic interaction outputs (`ChartOutput`),
- optional cross-chart propagation via `LinkConfig.group` + `ChartOutput.link_events` (ADR 1146),
- multi-grid routing contracts (ADR 1171) where any grid-tied output must be unambiguous (`GridId` in output).

Commercial-grade chart surfaces also require **cursor and axisPointer linking** across:

- multiple charts in a dashboard (crosshair sync),
- multiple grids inside one chart surface (multi-grid sync),
- and potentially multiple windows (future).

Without a contract, adapters risk propagating pixel positions or UI-specific details (DPI/layout dependent),
leading to silent drift and hard-to-fix breaking changes.

## Relationship to Other ADRs

- ADR 1146: link events for brush selection (change-based emission).
- ADR 1171: multi-grid viewport/layout and routing invariants (`GridId` must be carried for routing).
- ADR 1148: tooltip formatting contract (cursor/axisPointer affects tooltip sampling).

## Decision

### 1) Link events for cursor/axisPointer are **data-domain anchored**, not pixel-domain

Link payloads must be stable across:

- different viewports/layout policies,
- different DPI/scale factors,
- different renderers.

Therefore, the engine emits a link event anchored in the data domain.

The payload is a stable anchor:

- `AxisPointerAnchor { grid, axis_kind, axis, value }`

Where:

- `grid: Option<GridId>` is the resolved grid for routing (multi-grid).
- `axis_kind: AxisKind` is `X` or `Y`.
- `axis: AxisId` identifies the axis in the chart model.
- `value: f64` is the value in axis domain coordinates.

And the link event uses the anchor as an optional payload so hosts can also clear remote crosshairs:

- `LinkEvent::AxisPointerChanged { anchor: Option<AxisPointerAnchor> }`

Notes:

- For `trigger=Axis` tooltips, the event should prefer the trigger axis.
- For `trigger=Item` interactions, the engine may still emit axisPointer events when a hit exists
  (e.g. to sync crosshair to the hit’s `x_value`/`y_value`).

### 2) Events are change-based and gated by link configuration

When `LinkConfig.group` is `Some(_)`, the engine emits an axisPointer link event when the effective anchor changes:

- no per-frame spam for stable pointer positions,
- no duplicate emission when `step()` runs multiple times per frame.

Clearing semantics:

- when the axisPointer becomes inactive (e.g. pointer leaves the plot viewport), emit
  `AxisPointerChanged { anchor: None }` once (change-based), so linked charts can clear.

### 3) The link event stream is propagation-only; outputs remain the source of truth

The engine’s authoritative outputs remain:

- `ChartOutput.axis_pointer` (already carries `GridId`),
- `ChartOutput.hover` (when applicable).

Link events are intended for host-level routing to other charts in the same group.

## Adapter Responsibilities

UI adapters:

- continue to map pointer input into `Action::HoverAt { point: Px }` (pixel input),
- may additionally apply a received `AxisPointerChanged` event by converting it into an engine-facing action
  (follow-up: define a dedicated action surface for linked axisPointer anchors).

## Consequences

- Hosts can implement crosshair sync without sharing UI layout or pixel coordinates.
- Multi-grid remains deterministic and debuggable (grid-addressable events).
- Future extensions can add richer payloads (e.g. `raw_index` identity) without breaking the baseline.

## Follow-ups

- Add an explicit action surface for applying a linked axisPointer anchor (to avoid UI-only hacks).
- Add a minimal demo that shows cross-chart crosshair sync via `LinkConfig.group`.
