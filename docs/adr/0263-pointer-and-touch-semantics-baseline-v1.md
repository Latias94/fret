# ADR 0263: Pointer and Touch Semantics Baseline (v1)

Status: Proposed

## Context

Fret targets editor-grade interaction, but it also intends to support future mobile platforms.
Mobile input is rewrite-prone unless the framework locks the “hard invariants” early:

- pointer identity (multi-touch),
- per-pointer capture and cancellation,
- click vs drag normalization (click slop),
- hover availability boundaries (touch vs mouse),
- and where gesture policy is allowed to live (mechanism vs ecosystem policy).

The project already has several focused contracts:

- Pointer identity and per-pointer capture: ADR 0150 / ADR 0151.
- Click count normalization: ADR 0136.
- Coordinate mapping under transforms: ADR 0238 (and ADR 0082).
- Move coalescing + snapshots: ADR 0243.

This ADR consolidates the **mobile baseline** expectations and makes the mechanism/policy boundary
explicit, so component ecosystems do not accidentally encode desktop-only assumptions.

## Goals

1. Define well-formed pointer stream invariants (Down/Move/Up/Cancel) for touch and mouse.
2. Lock per-pointer capture/cancel semantics as the runtime mechanism baseline.
3. Lock click-vs-drag normalization as runner responsibility (no ad-hoc widget timing).
4. Define “hover is optional” as a contract boundary for mobile readiness.

## Non-goals (v1)

- A full Flutter-style gesture arena in the runtime.
- Axis-lock / inertial scrolling / long-press behaviors (policy belongs in ecosystem crates).
- Prescribing how scroll containers compete with child pressables (policy).

## Decision

### D1 — Pointer streams are well-formed and pointer-id scoped

For each `PointerId`, the runner MUST produce a well-formed stream:

- `Down` begins a contact/session.
- `Move` may occur zero or more times after `Down`.
- The stream ends with exactly one of:
  - `Up`, or
  - `PointerCancel`.

After `Up` or `PointerCancel`, no further events for that `PointerId` may be emitted until a new
contact begins (and the id may be reused only after the prior stream ends).

Rationale: ecosystem policies (drag sessions, docking, viewport capture, scroll) require deterministic
session ownership.

### D2 — Capture is per-pointer and cancellation always breaks capture

Pointer capture is defined per pointer (ADR 0150):

- capture maps `PointerId -> NodeId`,
- capture affects routing only for that pointer,
- captured pointer positions are mapped into the capturing node’s layout space (ADR 0238),
  and MUST NOT be clamped to bounds (sliders/drags rely on out-of-bounds coordinates).

When `Event::PointerCancel` is received:

- the runtime MUST clear capture for that `PointerId`,
- and component-owned cancel hooks MUST run (best-effort) so policy layers can revert state.

### D3 — Hover is capability-gated and must not be assumed

Touch-first platforms frequently do not produce hover.

Contract:

- Components MUST NOT assume “Move with no buttons” exists for the primary pointer.
- Hover-driven recipes (tooltips, hover cards, hover intent) MUST be gated on capability/environment
  queries (ADR 0232) or platform capabilities (ADR 0054).

Policy note:

- Synthesizing “touch hover” (e.g. long-press pre-hover) is allowed but is ecosystem-owned policy.

### D4 — Click vs drag is normalized by the runner (logical px)

The runner computes click normalization and exposes it through core pointer events:

- `PointerEvent::Down.click_count`
- `PointerEvent::Up.is_click`
- `PointerEvent::Up.click_count`

Semantics (ADR 0136):

- `click_count` increments only for “true clicks” (press + release within click slop).
- `is_click` indicates whether the current press/release qualified as a click, even if the click
  sequence count does not advance.
- Click slop is defined in **logical pixels** (ADR 0017).

Consequence:

- Widgets and ecosystem policies MUST NOT implement their own click timing/slop logic unless they
  intentionally want a different behavior and can justify it.

### D5 — Gesture policy lives in ecosystem crates

To keep `crates/fret-ui` mechanism-only (ADR 0066), higher-level touch policies SHOULD live in
ecosystem crates, for example:

- pan threshold and axis lock,
- capture-steal policies,
- inertia/fling,
- long-press, hover intent, and press-and-hold affordances.

The runtime’s responsibility is to provide:

- correct routing, capture, cancellation,
- deterministic coordinates,
- and stable capability/environment query seams.

## Consequences

- Mobile readiness is improved without committing to a specific gesture framework early.
- Component ecosystems can stay portable by capability-gating hover-only affordances.
- Complex interactions (docking, DnD, viewport tools) can evolve to multi-pointer semantics without
  retrofitting identity/capture later.

## References

- ADR 0136: `docs/adr/0136-pointer-click-count-and-double-click.md`
- ADR 0150: `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`
- ADR 0151: `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`
- ADR 0232: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- ADR 0238: `docs/adr/0238-pointer-coordinate-spaces-and-element-local-mapping-v1.md`
- ADR 0243: `docs/adr/0243-pointer-motion-snapshots-and-move-coalescing-v1.md`

