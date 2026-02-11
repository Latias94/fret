# ADR 0215: Scroll Handle Revision-Based Invalidation Contract

Status: Accepted

Note: Superseded by ADR 0217 (offset-only invalidation + children-only scroll transforms).

## Context

Fret uses a declarative, per-frame element tree (ADR 0028) with a retained `UiTree` substrate (ADR 0005). Some UI behavior is driven imperatively by small handles stored in component-layer code
(e.g. scroll handles, list scroll-to-item helpers).

If imperative handle changes are not coupled to the `UiTree` invalidation model, the framework can
enter a "nothing is dirty" state where:

- an imperative state change is recorded (e.g. `scroll_to_item` is queued),
- a frame is rendered (for unrelated reasons),
- but layout/paint does not re-run the subtree that needs to consume the change,
- resulting in "imperative calls sometimes do nothing" and a degraded authoring experience.

Zed/GPUI addresses similar problems by explicitly staging state and using `notify()` / dirty sets
to ensure that relevant views are invalidated when imperative or model-driven changes occur.

Fret needs a small, explicit contract for how imperative scroll handles participate in invalidation
without pulling policy-heavy behavior into `crates/fret-ui`.

## Decision

### 1) Scroll handles carry an internal monotonic `revision`

`ScrollHandle` maintains an internal `revision` counter that increments when its effective scroll
state changes (offset, viewport size, content size) or when it is explicitly "touched" by higher
level helpers (e.g. virtual list deferred scroll requests).

The `revision` value is:

- cheap to read (`Copy`),
- stable across clones of the handle,
- used only for invalidation (not for correctness-critical identity).

### 2) Declarative mount records scroll-handle bindings per frame

During declarative mounting, the runtime records per-window bindings:

- `handle_key`: identity of the handle instance (pointer-stable key),
- `revision`: current revision,
- `element`: the element ID that references the handle.

### 3) The runtime invalidates bound nodes when a handle revision changes

Before the final layout pass, `UiTree` compares the current vs previous per-handle `revision`.
For each handle whose revision changed, the runtime invalidates all nodes bound to that handle with
`Invalidation::Layout`.

This guarantees that programmatic scroll changes and deferred scroll requests are consumed by the
layout pipeline even when there are no other invalidation triggers.

### 4) Virtual list scroll-to-item requests must participate in the same mechanism

`VirtualListScrollHandle::scroll_to_item` is defined as a layout-affecting change:

- it must ensure the base `ScrollHandle` revision changes,
- so the binding-based invalidation mechanism can trigger the required layout work.

## Consequences

- Imperative scroll operations become deterministic: "call -> next frame consumes the request".
- The runtime no longer needs to scan visible subtrees each frame to discover deferred requests.
- The mechanism is portable and remains policy-free (no scheduling or animation semantics implied).

Non-goals / known limits:

- This ADR does not require scroll handles to schedule redraw by themselves. Call sites must still
  request a new frame when performing imperative changes (via app/window scheduling or event
  handling).

## References

- ADR 0005: Retained UI tree: `docs/adr/0005-retained-ui-tree.md`
- ADR 0028: Declarative elements and element state: `docs/adr/0028-declarative-elements-and-element-state.md`
- ADR 0051: Model observation and invalidation propagation: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
- GPUI window invalidation patterns (non-normative): `repo-ref/zed/crates/gpui/src/window.rs`
