# ADR 0231: Container Queries and Frame-Lagged Layout Queries (v1)

Status: Accepted

## Context

Fret’s declarative authoring model is intentionally aligned with Tailwind/shadcn composition
outcomes, but Fret is not a DOM/CSS runtime (ADR 0066). As the ecosystem grows, "responsive"
variants increasingly show up in upstream sources:

- Tailwind viewport breakpoints (`sm:`, `md:`, ...).
- Container queries (`@container`, `@md/foo`) where a component adapts to the **width of its local
  container**, not the global window.

Today, Fret approximates container-query-driven recipes by using viewport-width breakpoints in
component code (e.g. `>=768px` for `md`). This is an acceptable bootstrap, but it will drift
quickly once docking/panels become the default UX:

- In editor-grade UIs, panels resize independently of the window (docking splits, collapsible
  sidebars, inspector panes).
- The same component is frequently reused in different containers with different widths.

To avoid a future rewrite, we want a stable, typed way for components to adapt to container size
without importing a CSS cascade model into the runtime.

Constraints:

- Layout must remain deterministic and non-reentrant (ADR 0113).
- The runtime must remain mechanism-only; container query policy belongs in ecosystem crates
  (ADR 0066).
- The contract must be portable across native + wasm runners.

## Decision

### D1 — Container queries are expressed as *layout queries* over committed geometry

The runtime provides a **layout query** mechanism: components can read *committed* bounds of a
named "query region" (typically an ancestor container) and branch their declarative tree based on
that geometry.

Key rule:

- Layout queries are **frame-lagged**: they observe the last committed layout snapshot, never
  partial layout state during the current frame’s layout solve.

Rationale:

- Prevents layout/build recursion ("layout depends on a query that depends on layout").
- Keeps the contract compatible with view caching and non-reentrant measurement.

### D2 — The runtime does not implement "container query policy"

`crates/fret-ui` does not define Tailwind breakpoint tables, hysteresis rules, or "responsive
variants". It only provides:

- a way to **mark** a subtree as a query region with stable identity,
- a way to **read** the region’s committed bounds,
- dependency tracking so queries participate in invalidation (see D4).

Policy lives in:

- `ecosystem/fret-ui-kit`: typed container-query helpers and recommended hysteresis defaults,
- `ecosystem/fret-ui-shadcn`: recipes that mirror upstream `@md/*` behavior.

### D3 — Query regions have stable identity and diagnostics-friendly names

The runtime supports creating a query region using a stable, explicit identity:

- Stable per-element identity: `GlobalElementId` (ADR 0028) + explicit keys when needed.
- Optional string name for diagnostics (not a stable ID): useful in inspector/log output.

The runtime must be able to report (in debug/diagnostics):

- last committed bounds for each region,
- which views/elements observed which region (best-effort is acceptable in v1).

### D4 — Layout queries participate in invalidation

If a view reads a layout query result, changes to that region’s committed bounds must be able to
trigger a rebuild of the dependent view subtree.

Contract requirement:

- Observing a query region is treated like observing a `Model<T>`: when the region's bounds change
  beyond a small epsilon, dependents are invalidated.

Notes:

- The invalidation threshold must be DPI-safe (logical px) and stable (avoid rebuild storms due to
  subpixel jitter).
- Implementations may debounce/coalesce invalidations per frame.

### D5 — Recommended policy: breakpoint hysteresis to avoid oscillation

Container query consumers must assume that branching can change layout, and layout can change the
measured container width (especially in flex/grid contexts). To avoid oscillation, ecosystem-layer
helpers should provide hysteresis:

- Switch "up" at `>= threshold + hysteresis_px`
- Switch "down" at `< threshold - hysteresis_px`

The runtime does not enforce this, but the ecosystem should treat it as a best practice.

### D6 — Non-goals (v1)

This ADR explicitly does **not** introduce:

- A stringly Tailwind/CSS class parser.
- A CSS selector/cascade system.
- Arbitrary style rules applied by selectors.
- "One-pass, same-frame" container queries.

## Consequences

- Components can implement upstream container-query-driven recipes without hard-coding viewport
  breakpoints, which is critical for docking/panel-heavy editor UIs.
- The runtime remains mechanism-only and portable.
- We accept a 1-frame adaptation latency as the price of determinism and non-reentrancy.

## Implementation Notes (non-normative)

One plausible shape (details intentionally left open):

- A pass-through wrapper element kind (e.g. `LayoutQueryRegion`) that records its committed bounds.
- A small query API (e.g. `bounds_for_query_region(id) -> Option<Rect>`) exposed to declarative
  authoring and ecosystem helpers.
- Dependency tracking: recording "region X was read by view root Y" so a bounds change can trigger
  a view rebuild.

## References

- Mechanism-only runtime boundary: ADR 0066 (`docs/adr/0066-fret-ui-runtime-contract-surface.md`)
- Declarative layout semantics: ADR 0057 (`docs/adr/0057-declarative-layout-style-and-flex-semantics.md`)
- Tailwind layout primitives: ADR 0062 (`docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`)
- Non-reentrant measurement: ADR 0113 (`docs/adr/0113-available-space-and-non-reentrant-measurement.md`)
- Docking and editor-grade panels: ADR 0013 / ADR 0011 (`docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0011-overlays-and-multi-root.md`)

