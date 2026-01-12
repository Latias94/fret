# ADR 0075: Docking Layering (B Route) and Retained Bridge

Status: Accepted (foundation + migration plan)

## Context

Docking is an "editor-grade UX" capability, but it is also one of the easiest places for
application/editor-specific policy to leak into the UI runtime.

We previously had docking UI and viewport overlay details implemented in `crates/fret-ui` via a
retained-widget implementation (`DockSpace` lived in `crates/fret-ui/src/dock.rs`). This was
convenient early on, but it conflicts with:

- ADR 0027: framework scope vs editor app responsibilities
- ADR 0066: keep `fret-ui` as a small, optimizable contract surface
- ADR 0074: keep interaction policy component-owned, runtime mechanism-only

If we keep growing the docking UI implementation inside `fret-ui`, we risk locking editor-specific
policy (viewport gizmos/selection/marquee overlays) into the runtime, making future portability and
performance work harder.

We choose the **B route**:

- Keep docking **data model + ops + persistence shapes** as stable contracts (in `fret-core`).
- Move docking **UI + interaction policy** into a dedicated crate outside `fret-ui`.
- Keep `fret-ui` focused on **mechanisms** (event routing, layering, performance substrate).

## Decision

### 1) Layering split

**`crates/fret-core` (Stable contracts)**

- `DockGraph`, `DockNode`, `DockOp`, `DropZone`, persistence shapes and IDs.
- No UI rendering, no event routing, no viewport overlay policy.

**`crates/fret-ui` (Mechanism-only runtime substrate; ADR 0066)**

- Hit testing, focus/capture, overlay roots/layers, layout/paint substrate.
- Viewport embedding contracts (render targets + input forwarding boundary).
- Generic internal drag routing extension point (see below).
- Transitional, feature-gated compatibility APIs only, with explicit delete plans.

**New: `ecosystem/fret-docking` (Policy-heavy docking UI)**

- DockSpace UI composition (tabs/splits/tear-off interactions).
- Dock-specific drag preview visuals and policy.
- Depends on `fret-core` dock contracts + `fret-ui` mechanism substrate.
- May use a feature-gated retained-widget bridge during migration (see below), but must not leak
  retained authoring into shadcn/tailwind components.

**`crates/fret-editor` / app layer (Editor-specific viewport policy)**

- Viewport overlays such as gizmos, marquee, selection rects, markers.
- Tool routing, picking, and camera navigation policy (ADR 0049).

### 2) Generic internal-drag routing (mechanism)

Some cross-window drag flows must route internal drag events to a stable “anchor” node even when
hit-testing fails (e.g. docking tear-off).

`fret-ui` provides a mechanism-only table:

- `fret_ui::InternalDragRouteService` (per window + drag kind → dispatch root node)

This avoids hard-coding dock-specific concepts into the runtime.

### 3) Retained bridge (transitional; feature-gated; explicitly unstable)

Docking UI is currently implemented as a retained widget. To migrate it out of `fret-ui` without a
large rewrite, `fret-ui` provides a **feature-gated retained bridge**:

- Feature: `fret-ui/unstable-retained-bridge`
- Module: `fret_ui::retained_bridge`
- Scope: expose only the minimal retained-widget authoring surface needed by the docking crate.
- Explicitly **unstable** and **not part of the stable runtime contract surface** (ADR 0066).

The goal is to enable a staged migration:

1. Move `DockSpace` out to `fret-docking` with minimal code churn.
2. Keep `fret-ui` free of docking UI and viewport overlay policy.
3. Later, decide whether docking UI should remain retained (in docking crate) or be re-authored as
   declarative elements once the required declarative primitives exist.

## Migration Plan

### Stage 0 (this ADR)

- Add `InternalDragRouteService` (mechanism) in `fret-ui`.
- Add `unstable-retained-bridge` (feature-gated) in `fret-ui`.
- Migrate docking UI out of `fret-ui` into a dedicated crate (Stage 1), removing the in-runtime
  docking widget implementation from the runtime substrate.

### Stage 1 (move docking UI out)

- Create `ecosystem/fret-docking`.
- Move `DockSpace` and related UI/policy code out of `crates/fret-ui/src/dock.rs` (done; now in `ecosystem/fret-docking/src/dock/space.rs`).
- Register internal drag routing via `InternalDragRouteService`.

### Stage 2 (thin the runtime further)

- Remove `experimental-docking` module from `fret-ui`.
- Audit any remaining dock-specific routing/logic in `UiTree` and replace it with generic hooks.

### Stage 3 (editor overlays)

- Move viewport overlay shapes/policy (gizmo/marquee/selection) out of docking UI into `fret-editor`
  or app-layer code (ADR 0027 / ADR 0049).
- Provide an app-owned hook entry point for painting viewport overlays without re-introducing editor
  policy into docking UI:
  - `ecosystem/fret-docking`: `DockViewportOverlayHooks` + `DockViewportOverlayHooksService`
  - `crates/fret-editor`: `viewport_overlays` (reference shapes + paint helpers)

### Exit criteria (clean state)

- `crates/fret-ui` contains no docking UI widgets.
- `crates/fret-ui` contains no viewport overlay policy/shape enums.
- Docking UI is implementable without runtime-owned policy shortcuts (ADR 0074).
- shadcn/tailwind component crates do not enable `unstable-retained-bridge`.

## Consequences

Pros:

- Keeps the runtime contract surface small and optimizable (ADR 0066).
- Allows docking UI to iterate rapidly without "polluting" runtime public APIs.
- Unblocks cross-platform portability work by keeping editor-specific policy out of the runtime.

Cons / Risks:

- The retained bridge is a sharp tool: it must remain feature-gated and treated as unstable.
- Docking migration will temporarily duplicate patterns between retained and declarative layers.

## References

- ADR 0027: Framework scope and responsibilities
- ADR 0066: `fret-ui` runtime contract surface
- ADR 0072: Docking interaction arbitration matrix
- ADR 0074: Component-owned interaction policy and runtime action hooks
