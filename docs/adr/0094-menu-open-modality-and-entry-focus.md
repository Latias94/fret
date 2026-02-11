# ADR 0094: Menu Open Modality and Entry Focus

Status: Proposed

## Context

Radix `Menu` distinguishes whether the user is interacting via keyboard or pointer (see
`repo-ref/primitives/packages/react/menu/src/menu.tsx`, `isUsingKeyboardRef`). This drives a key
focus policy:

- On open, the menu content itself is focused.
- When the menu was opened via pointer, Radix prevents the “entry focus” behavior that would
  otherwise move focus to the first menu item (`onEntryFocus` + `preventDefault()`).
- When the menu was opened via keyboard, moving focus to the first enabled item is the expected
  APG/Radix outcome.

In Fret today, menus are implemented as dismissible overlay roots (ADR 0067 / ADR 0069). When an
overlay opens, the overlay controller applies initial focus via
`fret_ui_kit::primitives::focus_scope::apply_initial_focus_for_overlay`:

- If `OverlayRequest.initial_focus` is set, we try to focus that element.
- Otherwise, we fall back to the first focusable descendant under the overlay root.

Our menu wrappers (shadcn-aligned `DropdownMenu`, `ContextMenu`, `Menubar`) previously relied on
the fallback behavior, which means pointer-opened menus typically focus the first item. This
differs from Radix and can feel like “focus stealing” when the user is just pointing at a menu.

## Decision

### 1) Track a lightweight per-window input modality signal

Add a small per-window “last input modality” state in `crates/fret-ui`:

- `Keyboard` when the last observed event is `Event::KeyDown`.
- `Pointer` when the last observed event is a pointer/drag event.

This signal is intentionally minimal: it is a policy hint that component-layer behaviors can
consult without threading an explicit “open reason” through every model.

### 2) Gate menu overlay initial focus by modality (Radix-aligned outcomes)

Menu-like overlays gate `OverlayRequest.initial_focus` based on the last observed modality:

- **Keyboard modality**: `initial_focus = None`, allowing the overlay focus helper to focus the
  first focusable descendant (typically the first enabled menu item).
- **Pointer modality**: `initial_focus = Some(menu_content_container_id)`, focusing the roving
  container/content node while leaving the active item unset (Radix “prevent entry focus”).

## Notes / Limitations

- This is not an accessibility (a11y) solution and does not attempt to detect assistive
  technologies. It is a pragmatic alignment mechanism for early UI behavior, with explicit future
  room for a11y integration.
- When the window is unknown, or when a menu is opened programmatically without a preceding input
  event, the default modality is treated as `Pointer`.
- The modality state is per-window to support multi-window apps and to avoid cross-window policy
  bleed.

## Alternatives Considered

### Thread an explicit “open reason” through every menu model

Pros:

- Fully explicit at call sites.

Cons:

- High boilerplate and easy drift across component surfaces.
- Harder to apply consistently to nested menus/submenus.

### Always focus the content container (never focus the first item)

Pros:

- Avoids pointer-open focus stealing.

Cons:

- Breaks the expected keyboard-open behavior and APG/Radix outcomes.

## Consequences

- Menu opening behavior matches Radix more closely: pointer opens avoid focusing the first item,
  keyboard opens focus the first enabled item.
- The modality signal can be reused by other interactive primitives that need similar policy
  splits (e.g. focus-visible-ish behaviors at the component layer).

## Implementation Notes

- Menu triggers in this repo are typically `Pressable`-based and may request pointer capture on
  `PointerDown` to keep pressed state stable. Menu-like overlays must therefore not “auto-close”
  solely because there is an active pointer capture in another layer; instead, the overlay system
  should temporarily suppress pointer occlusion (Radix `disableOutsidePointerEvents`) while capture
  is active and then re-enable it once capture is released.

