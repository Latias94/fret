# ADR 0168: Viewport Tooling Host Helpers and Arbitration (v1)

Status: Proposed
Scope: Ecosystem-level integration helpers (no kernel contract changes)

## Context

Fret’s viewport boundary is intentionally Tier A and host-driven:

- UI embeds an engine-owned render target via `ViewportSurface` (ADR 0007).
- Pointer/wheel input is forwarded as `Effect::ViewportInput(ViewportInputEvent)` with explicit units (ADR 0147).
- Engine-pass 3D overlays (gizmos, debug draw, selection outlines) are recorded through a runner hook service
  (`ViewportOverlay3dHooksService`), consistent with the “engine owns the pass topology” boundary (ADR 0038 / ADR 0139).

This architecture keeps the kernel portable and avoids coupling viewport tools to `wgpu`/`winit`, but it introduces a
recurring integration problem:

- Each app/demo re-implements the same glue:
  - mapping `ViewportInputEvent` into tool-friendly coordinates (target px vs screen px),
  - stable drag/cancel phases,
  - cursor-unit scaling (DPI + fit mode),
  - “tool arbitration” (camera navigation vs selection vs gizmo) and capture rules,
  - wiring tool draw data into engine-pass overlay hooks.

Today we have:

- A tool-specific helper in `fret-gizmo` (`ViewportToolInput`) that standardizes input mapping for gizmos.
- A tool-specific plugin boundary in `fret-gizmo` (ADR 0155) and a host property read boundary (ADR 0167).
- A Deferred, editor-layer example ADR for tool managers and overlays (ADR 0049).

However, viewport tooling is not just gizmos:

- 3D plot widgets (`fret-plot3d`) may need orbit/pan, picking, selection rectangles, and debug overlays.
- Future ecosystems (measurement rulers, light/camera gizmos, physics/nav visualization, scene selection tools) will need
  the same Tier A glue and arbitration.

We want to reduce duplication and drift while preserving the kernel boundaries.

## Goals

- Provide a **single recommended integration story** for Tier A viewport tooling, usable by multiple ecosystem crates.
- Keep tool libraries portable:
  - no direct dependency on `wgpu`/`winit`,
  - avoid depending on `fret-gizmo` just to reuse generic input mapping utilities.
- Keep the kernel stable:
  - no new `fret-core` / `fret-runtime` contracts in v1.
- Make “tool arbitration” explicit and composable:
  - apps can override policy, but the default helper should be good enough for small apps and demos.

## Non-goals

- Defining a fully general editor tool system (document registry, selection model, undo policy) inside framework crates.
- Replacing engine-owned picking with a framework-owned picking runtime.
- Introducing a global “viewport tool manager” into `fret-app` in v1.

## Decision

Introduce ecosystem-level **viewport tooling host helpers** that standardize Tier A glue without changing kernel
contracts.

### 1) Define a portable “tool input mapping” helper (shared)

We standardize a small, unit-explicit input mapping helper that is usable by multiple tool ecosystems.

Minimum derived fields (conceptual):

- `viewport`: `ViewportRect` (caller-defined pixel space)
- `cursor_px`: cursor position in the same units as `viewport`
- `drag_started` / `dragging`: primary-button drag state
- `cursor_units_per_screen_px`: conversion factor for “screen logical px → cursor units”
- `cursor_over_draw_rect`: conservative “inside viewport” gate

This helper must be:

- portable (depends on `fret-core` and math types only),
- explicit about units (target px vs screen px),
- usable by non-gizmo tooling crates.

Placement options (to be decided during implementation):

- **Preferred**: a small policy-light ecosystem crate (e.g. `ecosystem/fret-viewport-tooling`) depending on
  `fret-core` + `glam`.
- Acceptable alternative: a module in `ecosystem/fret-ui-kit` if we want to keep ecosystem crates minimal, but note
  this would force headless tooling crates to depend on `fret-ui` indirectly (undesirable for some ecosystems).

### 2) Standardize “tool arbitration” as an optional helper (policy layer)

We provide an optional policy helper that turns a stream of `ViewportInputEvent`s into a stable “who is active”
decision:

- only one tool is “active” during a drag/capture,
- tools can be “hovered-enabled” only when they win arbitration,
- cancellation (`Esc`) is routed consistently.

This helper should be policy-driven and live in an ecosystem policy crate (typically `fret-ui-kit`), not the kernel.

The helper must support:

- app-defined precedence (e.g. camera nav has priority while RMB is down),
- modal gating (respect UI modal barriers; see ADR 0011 / ADR 0072),
- multi-tool coexistence (gizmo + selection + camera).

### 3) Keep engine-pass overlay integration explicit, but provide a recommended wiring shape

Tool rendering is intentionally engine-pass for Tier A:

- tools produce draw data (often 3D world-space lines/triangles) that the engine records into the viewport pass,
- the runner provides a stable hook point (`ViewportOverlay3dHooksService`).

We keep this boundary explicit but recommend a standard “registration” and “per-frame record” shape so multiple tool
ecosystems can participate without bespoke boilerplate.

### 4) Tool-specific read/write boundaries remain tool-owned

Some tools need to read or write domain values:

- gizmo plugins read host values via `GizmoPropertySource` (ADR 0167) and emit edits via `GizmoCustomEdit`.

This ADR does not force a universal “property system”. Instead:

- the shared host helper should provide a place to thread tool-specific property sources through,
- tool ecosystems define their own typed/validated write paths (host-owned apply + undo/redo).

## Consequences

### Benefits

- Ecosystem crates can share one “viewport tooling glue” story without coupling to each other.
- Demos and apps become smaller and less error-prone (fewer ad-hoc unit conversions).
- Arbitration policy becomes explicit and testable.

### Costs

- Another ecosystem surface to document and stabilize.
- Some ecosystems will still need custom policy (camera/nav constraints, selection semantics).

## Alternatives Considered

### A) Keep per-demo glue code (status quo)

Pros: no new surface.

Cons: duplication, drift, subtle unit/capture bugs repeated across ecosystems.

### B) Put tooling integration in kernel crates

Rejected for v1: likely to over-commit to an editor-specific policy surface and increase churn on “hard-to-change”
contracts.

### C) Put generic helpers inside `fret-gizmo`

Rejected: other ecosystems should not depend on `fret-gizmo` just to reuse input mapping utilities.

## Implementation Plan (Suggested)

1) Decide placement for the shared tool input mapping helper:
   - prefer `ecosystem/fret-viewport-tooling` (policy-light, `fret-core` + `glam`).
2) Move/duplicate `ViewportToolInput`-class logic into that shared helper and have `fret-gizmo` re-export it.
3) Add `fret-ui-kit` policy helpers:
   - default bindings for snap/precision/cancel,
   - a small arbitration state machine for camera vs selection vs gizmo.
4) Migrate `apps/fret-examples` viewports to the helper as the canonical reference.

## References

- Tier A viewport embedding: `docs/viewport-panels.md`
- Explicit-units viewport input: `docs/adr/0147-viewport-input-forwarding-explicit-units.md`
- Engine-pass gizmo boundary: `docs/adr/0139-viewport-gizmos-engine-pass-and-ui-overlay-boundary.md`
- Engine render hook substrate: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Gizmo plugin contract: `docs/adr/0155-custom-gizmo-plugins-and-handle-contract.md`
- Gizmo host properties (read-only): `docs/adr/0167-gizmo-host-property-source-readonly-v1.md`
- Editor-layer example (deferred): `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

