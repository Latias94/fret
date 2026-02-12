---
name: fret-action-hooks
description: Component-owned interaction policy in Fret (ADR 0074). Use when wiring press/hover/dismiss/roving/typeahead/timer behavior via `ElementContext` action hooks (e.g. `pressable_on_activate`, `dismissible_on_dismiss_request`, `roving_on_active_change`, `timer_on_timer_for`), or when migrating runtime-owned shortcut fields to component-layer hooks.
---

# Fret action hooks (policy lives in components)

Fret keeps `crates/fret-ui` **mechanism-only**. Interaction policy (what happens on click, Escape,
outside press, roving navigation, typeahead, focus restore…) lives in component crates via **action hooks**.

## When to use

Use this skill when:

- You need “what happens on activate/dismiss/roving/typeahead/timer” behavior in a component.
- You’re migrating legacy code away from runtime-owned policy fields on props.
- You’re debugging why a press/dismiss/typeahead behavior is inconsistent across components.

## Inputs to collect (ask the user)

Ask these before adding hooks (policy bugs are often “wrong scope” or “wrong ownership layer”):

- Which interaction family: press/activate, dismiss, roving focus, typeahead, timers?
- What is the policy state: which `Model<T>` (or `WeakModel<T>`) should change?
- What is the element scope where the hook must be installed (pressable scope, overlay content scope, etc)?
- What invariants must hold (e.g., menus are non-click-through, focus restores to trigger)?
- What regression gate is appropriate: unit test for policy vs diag script for event sequences?

Defaults if unclear:

- Keep policy in `ecosystem/`, install hooks in the narrowest correct scope, and add a `tools/diag-scripts/*.json` repro for state machines.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`

## Overview

**Why hooks exist:**

- Avoid contract churn in the runtime.
- Keep hot paths (dispatch/focus/overlay) optimizable.
- Make policy testable in `fret-ui-kit` / `fret-ui-shadcn`.

**Key surfaces (runtime plumbing):** `crates/fret-ui/src/action.rs` + `ElementContext` helpers.

**Component convenience helpers:** `fret_ui_kit::declarative::action_hooks::ActionHooksExt`.

## Quick start

### Toggle a `Model<bool>` on activation (recommended)

```rust
use fret_ui_kit::prelude::*;
use fret_ui::element::PressableProps;
use std::sync::Arc;

pub fn toggle_button<H: UiHost>(cx: &mut ElementContext<'_, H>, open: &Model<bool>) -> AnyElement {
    let open = open.clone();
    cx.pressable_with_id_props(|cx, _state, _id| {
        // Attach policy hooks within the Pressable scope.
        cx.pressable_toggle_bool(&open);

        let mut props = PressableProps::default();
        props.a11y.test_id = Some(Arc::from("toggle-open"));
        props.a11y.label = Some(Arc::from("Toggle"));

        (props, vec![ui::text(cx, "Toggle").into_element(cx)])
    })
}
```

If you are in component code, prefer the `fret-ui-kit` helper (less boilerplate, supports weak models):

- `cx.pressable_toggle_bool(&open_model)`
- `cx.pressable_toggle_bool_weak(&open_model.downgrade())`

### Close a dismissible overlay on outside press / Escape

Use `dismissible_on_dismiss_request` (policy) instead of baking dismissal into runtime widgets.

```rust
use fret_ui_kit::prelude::*;

pub fn install_dismiss_policy<H: UiHost>(cx: &mut ElementContext<'_, H>, open: &Model<bool>) {
    // Fret-ui-kit helper: close the open model on dismiss.
    cx.dismissible_close_bool(open);
}
```

### Roving selection writes (tabs, menu, listbox)

Roving focus movement is a runtime mechanism; *what selection means* is component-owned policy.

- Attach `roving_on_active_change` hooks to write selection models.
- Attach `roving_on_typeahead` hooks to implement matching rules (first-char or prefix buffer).

Use the `ActionHooksExt` helpers:

- `cx.roving_select_option_arc_str(&selected, values)`
- `cx.roving_typeahead_prefix_arc_str(labels, timeout_ticks)`
- `cx.roving_nav_apg()` (APG-aligned default navigation policy)

## Workflow (recommended checklist)

1. Decide the policy layer:
   - `fret-ui` owns *mechanisms* (routing, focus primitives, overlay roots).
   - Components own *policy* (toggle models, close overlays, selection writes).
2. Attach hooks inside the correct element scope:
   - Pressable hooks inside `cx.pressable_with_id_props(...)` or equivalent.
   - Dismiss hooks inside dismissible scopes / overlay content render scopes.
3. Prefer `WeakModel<T>` in long-lived callbacks (timers, delayed dismiss).
4. Add a regression gate:
   - Unit test for the headless policy when possible, or
   - `fretboard diag` script when event sequences matter.

## Definition of done (what to leave behind)

- Policy is expressed as hooks (no runtime flags/props that encode policy).
- Hooks are installed inside the correct element scope (they actually fire).
- Long-lived callbacks use `WeakModel<T>` where appropriate (no accidental “keep alive”).
- One regression artifact exists:
  - unit test for pure policy logic, and/or
  - `tools/diag-scripts/*.json` for multi-event sequences (dismiss/focus/roving/typeahead).

## Best practices

- Prefer `WeakModel<T>` for long-lived callbacks (timers, deferred close, hover intent) to avoid keeping state alive unintentionally.
- Keep “disabled” behavior consistent by using `*_if_enabled` command helpers where applicable:
  - `cx.pressable_dispatch_command_if_enabled(command)`
- When migrating old code:
  - remove runtime shortcut fields (toggle/set/select/typeahead) from props,
  - attach hooks during render instead.

## Evidence anchors (where to look)

- Runtime hook plumbing: `crates/fret-ui/src/action.rs`
- Component convenience helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`
- Docs:
  - Action hooks overview: `docs/action-hooks.md`
  - ADR 0074: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`

## Common pitfalls

- Capturing strong `Model<T>` in long-lived closures (leaks state / keeps overlays alive).
- Installing hooks outside the intended element scope (hook never runs).
- Re-introducing policy in runtime by “just adding a flag” (creates churn).

## Related skills

- `fret-component-authoring` (identity/state/invalidation patterns)
- `fret-overlays-and-focus` (dismiss/focus policy in overlay families)
- `fret-diag-workflow` (scripted interaction repros)
