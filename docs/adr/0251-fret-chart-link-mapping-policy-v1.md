# ADR 0251: `fret-chart` Link Mapping Policy v1 (Axis Keyed)

Status: Proposed (P0)

## Context

`delinea` emits link events for cross-chart coordination (ADRs 0207, 0249, 0250). These engine
events are intentionally **chart-local** and use `AxisId` / `GridId` for unambiguous routing inside
one chart model.

Dashboard-grade experiences require linking across charts that do **not** share the same `AxisId`
space (different specs, different generated IDs, different multi-grid topology).

We need a host-level mapping policy that:

- remains deterministic and portable (no pixel coordinates),
- avoids silent drift (no guessing unless explicitly requested),
- is incremental to adopt (explicit mapping first, auto mapping when unambiguous),
- and fits Fret’s layering: engine provides mechanisms; adapter/app provides interaction policy.

## Decision

### 1) Introduce a stable semantic axis key for linking

`fret-chart` defines a portable key:

- `LinkAxisKey { kind: AxisKind, dataset: DatasetId, field: FieldId }`

Where:

- `kind` is `AxisKind::{X,Y}` (from `delinea` spec/model),
- `dataset` / `field` are the originating dataset + encoded field driving that axis.

This aligns with the existing brush-derived cross-grid export policy (`SameDatasetXField`) while
remaining opt-in and policy-light.

### 2) Mapping policy is a host/adaptor responsibility (not engine responsibility)

The engine continues to emit:

- `LinkEvent::AxisPointerChanged { anchor: Option<AxisPointerAnchor> }` (ADR 0249),
- `LinkEvent::DomainWindowChanged { axis, window }` (ADR 0250),
- `LinkEvent::BrushSelectionChanged { selection }` (ADR 0207).

Hosts/adapters map chart-local IDs into stable keys and propagate state to other charts.

### 3) Mapping has an explicit-first, conservative fallback order

Given an incoming `AxisId`, the host resolves a `LinkAxisKey` using chart spec/model analysis.

Resolution order:

1. **Explicit axis map** (always wins)
   - A host may provide an explicit `AxisId -> LinkAxisKey` mapping table.
2. **Auto (ByAxisKey) mapping** (default)
   - If an axis can be uniquely associated with exactly one `(dataset, field)` for the axis kind,
     resolve that `LinkAxisKey`.
   - If ambiguous (multiple datasets/fields participate), do not link automatically.
3. **Best-effort heuristics** (optional, off by default)
   - Name/position-based matching is allowed only when explicitly enabled, and must be treated as
     non-contractual (best-effort).

### 4) Shared linking state is expressed in key space

To support charts with different `AxisId`, the shared linking state is keyed by `LinkAxisKey`, not
`AxisId`:

- AxisPointer: `Option<(LinkAxisKey, value: f64)>`
- Domain windows: `BTreeMap<LinkAxisKey, Option<DataWindow>>`
- Brush selection (v1): `Option<(x_key, y_key, x_window, y_window)>`

Adapters apply the shared state by mapping `LinkAxisKey -> AxisId` inside each chart (only when the
mapping is unique).

## Implementation Notes (v1)

`fret-chart` v1 provides:

- `ChartLinkRouter` to resolve `AxisId -> LinkAxisKey` and `LinkAxisKey -> AxisId`.
- `ChartLinkRouter::with_explicit_axis_map(...)` to override ambiguous auto mappings.
- `LinkedChartGroup` to propagate engine link events into shared key-space models while avoiding
  feedback loops.

Explicit mapping is expected to live in app/host code (or adapter configuration), not the engine.

## Consequences

- Linking remains stable under refactors that change `AxisId` values.
- Ambiguity is handled safely (no silent incorrect linking).
- The engine stays policy-light and portable.

## Follow-ups

- Add a conformance harness that links two different chart specs via `LinkAxisKey` and verifies:
  - axisPointer sync,
  - zoom/pan sync (domain windows),
  - brush sync (when axis keys are unambiguous).
