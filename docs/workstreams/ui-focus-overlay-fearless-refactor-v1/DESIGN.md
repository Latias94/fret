# Design

## Goals

- Prevent “flaky” overlay behavior under retained/view-cache reuse (parent pointers temporarily
  stale) without rewriting the entire UI runtime.
- Align overlay dismissal semantics with Radix/DOM-style contracts:
  - topmost non-modal dismissible overlay observes outside presses
  - branches (e.g. trigger) do not count as outside
  - policy may prevent default dismissal behavior
  - default side effects (e.g. focus clearing) must be policy-controllable
- Prepare a clean path to a GPUI-like per-frame dispatch snapshot (Phase C).

## Non-goals

- Redesign component policies (belongs in `ecosystem/*`).
- Change layering rules (`crates/fret-ui` remains mechanism/contract only).
- Large perf work (we only add small guardrails; perf is a separate workstream).

## Background (current hazards)

- Some containment checks rely on parent pointers.
- Retained/view-cache reuse can temporarily break or delay parent-pointer correctness.
- Some mechanisms already compensate by using child-edge reachability from active layer roots.
- Outside-press dismissal has a default focus-clearing side effect that must be policy-controllable.

## Phase A: Child-edge reachability for outside-press containment

### Problem

Outside-press routing decisions that depend on parent pointers can misclassify events when parent
pointers are stale (e.g. cache-root mount ordering, detach/reattach, reuse roots).

### Contract changes (mechanism-only, no policy change)

- Treat “is inside layer subtree” as **child-edge reachability** from active layer roots.
- Treat “is inside branch subtree” as **child-edge reachability** from the branch root.

### Acceptance criteria

- Outside press:
  - never dismisses when the hit target is inside the overlay subtree
  - never dismisses when the hit target is inside a registered branch
  - remains click-through when configured
- Behavior remains stable even if parent pointers are temporarily stale (synthetic tests).

## Phase B: Prevent-default should suppress default focus clearing

### Problem

Outside-press dismissal can apply a default focus-clearing side effect. If policy calls
`prevent_default()`, that default side effect must be suppressed to preserve Radix/DOM outcomes.

### Contract changes

- If the dismissible root’s outside-press handler calls `prevent_default()`, the runtime must not
  apply the default focus-clearing side effect for that outside-press interaction.

### Acceptance criteria

- When `prevent_default()` is applied for an outside press, focus remains unchanged unless the
  underlay’s hit-tested dispatch moves focus explicitly.

## ADR updates

- ADR 0069 is the contract surface for outside-press dismissal and branches. This workstream keeps
  the ADR up to date as Phase A/B land:
  - reachability-based containment (robust under stale parent pointers)
  - `prevent_default()` suppresses default focus clearing on outside press

## Phase C: Per-frame dispatch snapshot (GPUI-inspired)

### Intent

Make event routing and focus/tab containment authoritative to a single per-frame snapshot:

- dispatch tree (node containment / focus containment / bubble chains)
- hit-test structures (hitboxes + transforms)
- tab-stop list / roving focus metadata
- input-handler bindings (IME/text)

### Migration strategy

- Keep Phase A/B tests as invariants.
- Introduce snapshot behind a debug flag until parity is proven.
- Replace one subsystem at a time:
  1) outside-press containment + branches
  2) focus containment + focus scope trap
  3) tab order computation + traversal
  4) hit-test path caching + transforms

