---
title: "A11y Acceptance Checklist (Overlays + shadcn demo)"
---

# A11y Acceptance Checklist (Overlays + shadcn demo)

This document is a **manual accessibility acceptance checklist** for Fret’s window-scoped overlays
(Popover / ContextMenu) and their consumers (Select / Combobox) using the `fret-demo` UI kit harness.

The goal is to validate that:

- The **semantics tree** is produced (ADR 0033).
- Assistive technologies can **navigate items** (not just the overlay root).
- Assistive technologies can **invoke** the focused item (Click/Invoke) end-to-end.
- Overlay dismissal restores focus correctly (ADR 0067).

## Quick start (run the harness)

Run the shadcn gallery harness:

```bash
cargo run -p fret-demo --bin shadcn_gallery
```

Notes:

- The shadcn gallery requests a semantics snapshot every frame (`crates/fret-demo/src/shadcn_gallery.rs`).
- The desktop runner enables AccessKit integration by default via `WinitRunnerConfig.accessibility_enabled`.

## What to test (high signal checks)

### 1) Popover (Select)

Find the `Select (Popover overlay)` section and:

1. Focus the Select.
2. Open it (keyboard Enter/Space or click).
3. Use AT navigation to move through list items.
4. Invoke an item via AT “Click/Activate” action.
5. Verify the popover closes and focus returns to the Select trigger.

Expected semantics outcomes:

- Popover root is `List`.
- Items are exposed as `ListItem` with correct `disabled/selected` flags.

### 2) Context menu

Trigger a context menu (where available in the demo) and:

1. Open the menu.
2. Navigate items via AT.
3. Invoke an item via AT “Click/Activate”.
4. Verify the menu closes and focus returns to the previous focus target.

Expected semantics outcomes:

- Context menu root is `Menu`.
- Items are exposed as `MenuItem` with correct `disabled/selected/expanded` flags.

### 3) Combobox (typeahead + popover list)

In the UI kit combobox:

1. Focus the combobox input.
2. Open the list (ArrowDown or click).
3. Verify typing continues to edit the input while the list is visible.
4. Verify the combobox node reports `expanded=true` while the list is open.
5. Close the list and verify `expanded=false`.

Expected semantics outcomes:

- Combobox surface role is `ComboBox` (not only `TextField`).
- `value` reflects the current input text.
- `expanded` tracks whether the popover request exists for this owner.

## Recommended tooling (platform-specific)

### Windows

- Narrator (built-in) to validate navigation + invoke behavior.
- “Accessibility Insights for Windows” or “Inspect.exe” to inspect the UIA tree roles/flags.

### macOS

- VoiceOver (built-in).
- Accessibility Inspector to inspect the AX tree.

## Regression tests (keep these green)

These tests cover the “AT Invoke” paths by focusing the a11y child nodes and sending `Space`:

- Popover list item invoke closes overlay and sets result:
  - `crates/fret-components-ui/src/window_overlays.rs` (`popover_a11y_invoke_list_item_sets_result_and_closes`)
- Context menu item invoke closes overlay and dispatches command:
  - `crates/fret-components-ui/src/window_overlays.rs` (`context_menu_a11y_invoke_menu_item_dispatches_command_and_closes`)

Semantics presence checks:

- `crates/fret-components-ui/src/window_overlays.rs` (`popover_semantics_exposes_list_items`)
- `crates/fret-components-ui/src/window_overlays.rs` (`context_menu_semantics_exposes_menu_items`)
- `crates/fret-components-ui/src/window_overlays.rs` (`combobox_semantics_role_and_expanded_follow_popover`)

Run:

```bash
cargo nextest run -p fret-components-ui
```

## Known limitations / follow-ups

- Semantics schema is intentionally minimal (ADR 0033). It currently does not model combobox-to-list
  relationships (ARIA `controls/owns/activedescendant`-style linkage). If parity needs it, extend
  `fret-core` semantics schema behind an ADR + tests.
- Overlay item a11y nodes use a fixed slot pool (currently 256). Very long menus/lists will expose
  only the first N items to AT until a virtualized a11y contract is defined.
