---
title: "ADR 0115: Trigger Composition Without Slot/asChild (Rust-Native Ergonomics)"
---

# ADR 0115: Trigger Composition Without Slot/asChild (Rust-Native Ergonomics)

Status: Proposed

## Context

Radix UI Primitives (and shadcn/ui recipes built on top of them) rely heavily on `Slot` / `asChild`
to achieve two DOM-specific goals:

1. Attach interaction and a11y props to an arbitrary child element without adding an extra DOM
   wrapper node.
2. Compose event handlers and attributes (`className`, `style`, `aria-*`) across component
   boundaries while preserving authoring flexibility.

Fret is not a DOM runtime:

- Authoring nodes are typed `AnyElement` variants (e.g. `Pressable`, `TextInput`, `Semantics`),
  not untyped HTML elements.
- Interaction policy is attached via component-owned action hooks (ADR 0074) stored in element
  state for the element's stable `GlobalElementId` (ADR 0028/0039).
- Layering/portal behavior targets per-window overlay roots (ADR 0011, ADR 0067), not arbitrary DOM
  containers.

Attempting to port Radix `Slot` semantics 1:1 would either:

- require a generic "prop bag" system that collapses typed element contracts into a dynamic map, or
- require runtime-level support for "retargeting" action hooks and merging typed props across
  unrelated element kinds,

both of which would expand the `fret-ui` contract surface (ADR 0066) and add high risk for drift.

## Decision

We do **not** implement a general-purpose Radix-style `Slot/asChild` mechanism in Fret at this
time.

Instead, we standardize on a Rust-native trigger composition model:

1. **Triggers are explicit typed elements** (most commonly `Pressable`).
2. **Visual customization happens through trigger children** (icons/text/layout), not by swapping
   the trigger element kind.
3. **Policies are attached via action hooks** (`ElementContext::{pressable_*, pointer_region_*,
   dismissible_*}` and `fret-ui-kit::declarative::action_hooks`) rather than DOM event prop merging.
4. **A11y/relationship stamping is done by small helpers** that mutate a trigger node in-place
   when it is one of the expected element kinds (typically `Pressable` or `Semantics`), for example
   `apply_*_trigger_a11y(...)`.

## Authoring guidelines (recommended)

- If a Radix primitive has a `Trigger`, treat it as a `Pressable` in Fret.
- If you need a "link-like" trigger, provide a dedicated recipe (e.g. `LinkButton`) rather than
  relying on `asChild` to convert a button into an anchor.
- Prefer builder/closure-based APIs that keep the trigger type stable:
  - configuration on the Rust builder (`.disabled(...)`, `.variant(...)`, `.size(...)`)
  - children provided via closures (`|cx| vec![...]`)

## Consequences

- Fret primitives remain typed and predictable.
- Recipes stay small: they compose behavior by calling explicit helpers rather than relying on
  implicit prop merging.
- Some React authoring patterns (arbitrary element swapping without extra nodes) are intentionally
  not supported; the framework favors stable, Rust-native ergonomics.

## Future work

If a real need emerges (e.g. a wide class of recipes cannot be expressed ergonomically), we can
consider a **restricted** slot mechanism with explicit constraints, for example:

- only supports `Pressable` -> `Pressable` merging, and
- only composes well-defined fields (layout, enabled/focusable, a11y label/relations, and action
  hooks),

without introducing a dynamic prop bag or a runtime-level retargeting system.

## References

- Runtime contract surface and boundaries: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Component-owned action hooks: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Overlay policy architecture (Portal + Dismissal + Focus): `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Radix headless primitives alignment intent: `docs/adr/0089-radix-aligned-headless-primitives-in-fret-components-ui.md`

