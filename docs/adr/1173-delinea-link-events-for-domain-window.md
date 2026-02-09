# ADR 1173: `delinea` Link Events for Domain Windows (P0)

Status: Proposed (P0)

## Context

Interactive dashboards frequently require synchronized view windows:

- multiple charts aligned on X (time series zoom/pan),
- small “overview” chart driving a detailed chart (master/detail),
- multi-grid charts where grids are optionally coupled by policy.

`delinea` already has explicit view-window semantics (dataZoom + 2D view windows) and deterministic outputs,
but it does not yet standardize a link-propagation payload for window changes.

Without a contract, hosts and adapters will invent ad-hoc propagation logic, leading to inconsistent behavior and
hard-to-change surfaces.

## Relationship to Other ADRs

- ADR 1129: dataZoom X semantics and filter-mode behavior.
- ADR 1136: Y + 2D view semantics (mapping-first, constraints).
- ADR 1171: multi-grid routing invariants.
- ADR 1146: link events emission must be change-based and non-spammy.

## Decision

### 1) Domain window link events are explicit and axis-addressable

When `LinkConfig.group` is `Some(_)`, and a view window changes, the engine emits:

- `LinkEvent::DomainWindowChanged { axis, window }`

Where:

- `axis: AxisId` identifies the axis whose effective view window changed.
- `window: Option<DataWindow>` is the effective window (or `None` if cleared).

Notes:

- Multi-grid routing remains unambiguous through `AxisId -> GridId` in the model; implementations may optionally
  extend the payload with `grid: Option<GridId>` if it removes adapter-side lookups.

### 2) Events are change-based and de-duplicated across multi-step frames

The engine must:

- emit at most once per distinct window change,
- avoid re-emitting on subsequent `step()` calls unless the window changes again.

### 3) Default behavior remains per-grid scoped; cross-grid coupling is opt-in

This ADR does not introduce implicit coupling between grids.
Any cross-grid linking policy must be explicitly enabled (ADR 1171 section 5).

The minimal v1 policy is:

- if two axes have the same `AxisId` across charts (same spec family), hosts may propagate `DomainWindowChanged`
  directly,
- otherwise, hosts may apply a mapping policy (future ADR) such as `(dataset, encode.x)` or semantic tags.

## Consequences

- Hosts can implement “sync zoom/pan” with deterministic, portable payloads.
- UI adapters stay policy-light (gesture mapping only).

## Follow-ups

- Define an explicit axis mapping policy surface for cross-chart linking when `AxisId` differ.
- Add a conformance demo that links two charts’ X windows (and optionally Y) via `LinkConfig.group`.

