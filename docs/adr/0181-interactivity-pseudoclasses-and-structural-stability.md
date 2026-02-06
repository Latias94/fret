# ADR 0181: Interactivity Pseudoclasses + Structural Stability (Paint-Only by Default)

Status: Proposed

## Context

Editor-grade UIs have dense interaction chrome:

- hover highlights, focus rings, pressed states,
- transient toolbars (e.g. “copy” buttons),
- scrollbars that fade in/out on hover,
- list row hover/selection states at scale.

In a declarative per-frame element model (ADR 0028), it is easy to accidentally implement these effects by:

- adding/removing child nodes on hover,
- switching between different subtree shapes for “hovered vs not hovered”.

This causes:

- unnecessary layout invalidation and cache misses,
- visible flicker when cached subtrees are reused inconsistently,
- hard-to-debug “feels bad” regressions that scale with tree size.

GPUI’s pattern is to treat hover/focus/pressed as **pseudoclasses** that refine style, not structure:

- hover state is tracked by hitboxes,
- hover changes trigger `notify` (re-render),
- the element tree shape remains stable; only style refinements change.

This ADR locks those outcomes as a contract for Fret’s ecosystem.

## Decision

### 1) Define interactivity as pseudoclasses, not tree shape

Fret defines a set of runtime-managed interaction pseudoclasses:

- `:hover`
- `:focus`
- `:focus-visible`
- `:active` (pressed)
- `:disabled` (interactivity gate / disabled props)

These pseudoclasses are **inputs to style resolution** and MUST NOT require changing subtree structure to apply.

### 2) Structural stability rule (hard constraint)

For any component intended to be cache-friendly (ADR 1152) and editor-scale performant:

- hover/focus/pressed MUST NOT add/remove nodes,
- hover/focus/pressed MUST NOT switch between different element kinds at the subtree root,
- hover/focus/pressed MUST NOT alter child ordering.

Allowed mechanisms to express transient chrome:

- style refinements (colors, borders, shadows),
- `Opacity` (fade in/out),
- `InteractivityGate` (disable pointer/keyboard without unmounting),
- reserved layout space + “visual only” transitions (opacity/clip), not layout changes.

### 3) Invalidation default: paint-only

A pseudoclass transition (hover/focus/pressed) defaults to **paint-only** invalidation.

Escalation to layout invalidation is allowed only when the component explicitly opts in and documents why (e.g. a
discrete density mode toggle that changes intrinsic sizes).

#### Implementation hook (v1)

To make “paint-only by default” practical under view-cache reuse, pointer hooks need a way to request node-level
invalidations without forcing a rerender:

- `UiPointerActionHost::invalidate(Invalidation::Paint)` records a paint invalidation for the current pointer region /
  pressable node.
- Hooks should pair this with a redraw request (e.g. `host.request_redraw(action_cx.window)`) so the invalidation is
  observed on the next frame.
- `notify()` remains the escape hatch for structural/state changes that must rerender.

### 4) Cache boundary compatibility rule

When a subtree is behind a cache boundary (ADR 1152):

- pseudoclass-driven visuals MUST remain correct without requiring subtree re-execution.

This implies either:

- the cached output already includes both states in a stable structural way (e.g. always render the chrome but gate it
  with opacity/interactivity), or
- the pseudoclass styling is applied at a layer that still runs on cache hits (a future “paint-time style refinement”
  evolution).

### 5) Diagnostics and enforcement (debug-only)

To keep the ecosystem honest, the runtime/tooling SHOULD provide debug checks:

- count/layout invalidations triggered by hover transitions,
- warn when a subtree’s child list changes solely due to pseudoclass state,
- attribute offenders by `GlobalElementId` path (ADR 1151).

## Consequences

- Eliminates an entire class of hover-induced flicker/layout thrash bugs.
- Makes “view cache” (ADR 1152) practically usable in component-heavy surfaces.
- Encourages an authoring style that matches GPUI and scales to large editors.

## Alternatives Considered

### A) Allow structural changes on hover

Pros:
- simplest authoring in the small.

Cons:
- poor perf at scale; frequent cache busting; causes visible instability.

### B) Provide implicit “hover chrome” primitives in `fret-ui`

Pros:
- fewer footguns.

Cons:
- violates layering (ADR 0066) by baking policy into the runtime.

This ADR instead defines a runtime contract + tooling enforcement, while leaving policy to the ecosystem layer.

## References

- Zed/GPUI hover style refinement and hover-driven notify:
  - `repo-ref/zed/crates/gpui/src/elements/div.rs` (hover style refinement and `cx.notify(current_view)` on hover edge)
- Fret cache root semantics:
  - ADR 1152: `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
  - ADR 1152: `docs/adr/1163-view-cache-subtree-reuse-and-state-retention.md`
- Fret runtime contract surface and layering:
  - ADR 0066: `docs/adr/0066-fret-ui-runtime-contract-surface.md`

