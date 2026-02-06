---
name: fret-overlays-and-focus
description: Overlays and focus in Fret (Radix-aligned behavior). Use when implementing popovers/menus/tooltips/dialogs, wiring dismiss + focus trap/restore, debugging outside-press/click-through behavior, or using `OverlayController` / `OverlayRequest` / placement + anchoring helpers.
---

# Fret overlays and focus

Fret splits overlays into:

- **Mechanism (runtime):** overlay roots/layers, modal barrier, outside-press observers, placement solver
  (`crates/fret-ui`)
- **Policy (components):** dismissal rules, focus trap/restore, hover intent, presence/transition behavior
  (`ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`)

## Overview

**Key building blocks:**

- `OverlayController::{begin_frame, request, render}` (per-window coordination)
- `OverlayRequest` + convenience constructors:
  - `OverlayRequest::dismissible_popover(...)`
  - `OverlayRequest::dismissible_menu(...)` (non-click-through outside press; Radix-aligned)
  - `OverlayRequest::modal(...)`
- `OverlayPresence`: “present vs interactive” separation (avoid conflating with `open`)
- Focus policy hooks:
  - `on_open_auto_focus`, `on_close_auto_focus`
  - `initial_focus` (when you want deterministic first focus)

## Quick start: request a menu-like overlay (dismiss on outside press without click-through)

```rust
use fret_ui_kit::prelude::*;
use fret_ui::element::PressableProps;
use std::sync::Arc;

pub fn menu_example<H: UiHost>(cx: &mut ElementContext<'_, H>, open: Model<bool>) -> AnyElement {
    cx.pressable_with_id_props(|cx, _state, trigger_id| {
        // Read+observe open (so UI invalidates when it changes).
        let is_open = cx.get_model_copied(&open, Invalidation::Paint);

        // Policy: toggle the open model on activation.
        cx.pressable_toggle_bool(&open);

        if is_open {
            let overlay_id = cx.named("menu_overlay", |cx| cx.root_id());

            let req = OverlayRequest::dismissible_menu(
                overlay_id,
                trigger_id,
                open.clone(),
                OverlayPresence::instant(true),
                vec![ui::text(cx, "Menu content").into_element(cx)],
            );

            OverlayController::request(cx, req);
        }

        let mut props = PressableProps::default();
        props.a11y.test_id = Some(Arc::from("menu-trigger"));
        props.a11y.label = Some(Arc::from("Open menu"));

        (props, vec![ui::text(cx, "Open menu").into_element(cx)])
    })
}
```

## Behavior checklist (Radix-correct outcomes)

- **Dismiss**
  - Escape closes
  - outside press closes
  - trigger re-click closes
  - nested overlays (submenus) dismiss predictably
- **Click-through**
  - menus should usually be non-click-through on outside press (`dismissible_menu`)
  - simple popovers may allow click-through depending on policy (`dismissible_popover`)
- **Focus**
  - open: initial focus is deterministic (first item / configured `initial_focus`)
  - close: focus restores to the trigger (when appropriate)
- **Placement**
  - clamp/flip within constrained viewport
  - content becomes scrollable with max-height on tiny windows

## Debugging tips

- Capture a window snapshot after overlay synthesis:
  - `OverlayController::arbitration_snapshot(&ui)`
  - `OverlayController::stack_snapshot_for_window(...)`
- For scripted repros and bundles: use `fret-diag-workflow`.

## References

- Overlay architecture + policy split: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Outside press + non-click-through menus: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Focus traversal/scopes: `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Placement contract: `docs/adr/0064-overlay-placement-contract.md`
- Code entry points:
  - Runtime: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/overlay_placement/mod.rs`
  - Policy/controller: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
