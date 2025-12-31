# Action Hooks (Component-Owned Interaction Policy)

This document explains the “action hooks” mechanism and the layering rule introduced by ADR 0074:
**interaction policy belongs to components**, while `crates/fret-ui` stays a mechanism-only runtime.

## Problem Statement

Fret needs a small, stable runtime contract surface so we can aggressively optimize:

- event routing and input dispatch
- focus traversal / roving focus
- overlay roots + outside-press observers
- invalidation and rendering performance

If the runtime also “owns” UI kit policies (model writes, selection behavior, dismiss rules, typeahead
matching, etc.), we end up with:

- contract churn (every new component requires new runtime fields)
- duplicated policy in multiple widgets
- harder optimization (policy-specific branching in hot paths)
- weak separation between mechanism and design-system behavior

## Layering Rule

- `crates/fret-ui`: mechanisms + contracts only (tree, layout, hit-test, focus, semantics, overlay roots).
- `crates/fret-components-ui`: reusable infra (tokens, recipes, headless state machines).
- `crates/fret-components-shadcn`: shadcn/ui v4 naming surface (recipes; no retained widgets).

**Policy lives in the component layer**:

- “Toggle this model on activation”
- “Close this overlay on Escape or outside press”
- “Write selection when roving active changes”
- “Typeahead match rules (first-char vs prefix buffer)”

## What Is an Action Hook?

An action hook is a handler registered during declarative rendering, stored on the element node, and
invoked by the runtime when an interaction happens.

The runtime provides:

- the *trigger point* (e.g. “pressable activated”, “dismiss requested”, “roving active changed”)
- the *reason* (e.g. keyboard vs pointer, Escape vs outside press)
- a small action context (`ActionCx`) that can write models, dispatch commands, request focus, etc.

The component provides:

- the *policy* (what to do for the interaction)

## Hook Surfaces (Current)

Runtime plumbing is in `crates/fret-ui/src/action.rs` and exposed via `ElementCx` helpers:

- `ElementCx::pressable_on_activate(...)`
- `ElementCx::dismissible_on_dismiss_request(...)`
- `ElementCx::roving_on_active_change(...)`
- `ElementCx::roving_on_typeahead(...)`
- `ElementCx::pointer_region_on_pointer_down(...)` (for context menus, drag start, etc.)
- `ElementCx::pointer_region_on_pointer_move(...)` (for drag interactions and pointer capture streams)
- `ElementCx::pointer_region_on_pointer_up(...)` (for drag end / release semantics)
- `ElementCx::key_on_key_down_for(...)` (for keyboard-invoked context menus, custom key handling, etc.)

Component-layer convenience helpers live in:

- `crates/fret-components-ui/src/declarative/action_hooks.rs` (`ActionHooksExt`)

## Recommended Usage Patterns

### Reading Models During Rendering (Observe + Read)

Model invalidation is opt-in: if an element reads a model during rendering but does not register
observation, the runtime may not know it needs to invalidate layout/paint when the model changes.

Prefer `ElementCx` helpers that combine “observe + read”:

- `cx.get_model_copied(&model, Invalidation::Paint)` / `cx.get_model_cloned(&model, Invalidation::Layout)`
- `cx.read_model_ref(&model, Invalidation::Layout, |value| ...)`
- `cx.read_model(&model, Invalidation::Layout, |app, value| ...)`

### Toggle a Model on Activation

Prefer a component-layer helper:

- `ActionHooksExt::pressable_toggle_bool(model)`
- If the handler should not keep the model alive: `ActionHooksExt::pressable_toggle_bool_weak(model.downgrade())`

This attaches a `pressable_on_activate` hook, rather than using runtime shortcut fields.

### Close an Overlay on Dismiss

Overlays should be created via `fret-components-ui` overlay policy code (e.g. `window_overlays`).
Then register dismissal handlers via `dismissible_on_dismiss_request`:

- close on Escape
- close on outside press
- restore focus to a trigger when appropriate

### Roving Selection Writes

Roving navigation (Arrow/Home/End) is a runtime mechanism; “what it means” is component-owned.

Attach a `roving_on_active_change` hook to write selection models (tabs, radio, listbox, menu).

### Typeahead

Keep matching rules and buffer strategy in components:

- simplest: first-char match
- recommended: prefix buffer with timeout stored in element state

See `ActionHooksExt::roving_typeahead_prefix_arc_str(...)` and `headless/typeahead.rs`.

### Long-Lived Hooks: Prefer `WeakModel<T>`

Timer hooks and other long-lived callbacks should avoid capturing a strong `Model<T>` unless the
callback is intentionally responsible for keeping the model alive.

Prefer capturing `WeakModel<T>` and upgrading opportunistically, e.g. via:

- `fret_ui::action::UiActionHostExt::update_weak_model(...)`
- `ActionHooksExt::*_weak(...)` helpers in `crates/fret-components-ui`

## Transitional APIs

Some runtime-owned shortcut fields remain temporarily for compatibility, but are deprecated:

- `PressableProps` shortcut model writes (toggle/set variants)
- `RovingFocusProps` shortcut selection/typeahead fields

New/updated components should use action hooks instead.

## Design Goals (Why This Helps Optimization)

- **Performance:** hot-path runtime code stays generic; component policies do not add branching to
  input dispatch or focus navigation.
- **Extensibility:** new components can be built by composing headless helpers + hooks without
  expanding `fret-ui`’s public contract.
- **Testability:** policy logic lives in component crates where it can be unit-tested without
  touching the runtime.
- **Stability:** `fret-ui` remains small and portable across platforms and runners.

## Related Documents

- ADR 0074: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Overlay policy: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Outside press: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Focus scopes: `docs/adr/0068-focus-traversal-and-focus-scopes.md`
