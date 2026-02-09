# ADR 1146: `delinea` Link Events for Brush Selection (P0)

Status: Accepted (P0)

## Context

`delinea` supports an optional `LinkGroup` for cross-chart coordination (multiple charts reacting to the same
interaction intent). The headless engine already exposes a `link_events` queue in `ChartOutput`, but the surface
needs stable semantics before multiple UI adapters start depending on it.

Brush selection is a high-value interaction to link across charts (ECharts-style “brushLink”), especially for
commercial-grade dashboards and editor tools.

## Relationship to Other ADRs

- ADR 1144: brush selection as a headless output.
- ADR 1145: derived X-only row range fast path.

## Decision

### 1) Brush selection changes emit a link event when a LinkGroup is configured

When `ChartState.link.group` is `Some(_)`, the engine emits a link event whenever the brush selection changes:

- `LinkEvent::BrushSelectionChanged { selection: Option<BrushSelection2D> }`

The payload is the current selection (or `None` if cleared).

In multi-grid charts, `BrushSelection2D` carries an optional `GridId`, making the event grid-addressable without
requiring adapter-side guessing.

### 2) Events are change-based, not frame-based

The engine must not emit brush events every frame. Emission is gated by a cached last value:

- if the selection is unchanged, no event is emitted,
- if the selection changes (including from/to `None`), one event is emitted on the next `step()`.

This keeps the channel suitable for high-frequency pointer drags without introducing event spam.

Additionally, the engine may be stepped multiple times per frame (progressive rendering). Link events
must not be lost in that scenario: events accumulate in `ChartOutput.link_events` until drained by
the consumer.

### 3) Link events are a propagation mechanism, not the source of truth

Brush selection remains available as stable output fields (ADR 1144/0145). Link events are:

- intended for cross-chart propagation by the host app,
- optional and gated by link configuration,
- not required for single-chart usage.

## Consequences

- Hosts can implement ECharts-like `brushLink` by routing `BrushSelectionChanged` events to other charts in the
  same group.
- The semantics remain deterministic and cheap under large-data workloads.
