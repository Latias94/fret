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

Run the demo harness (shadcn gallery):

```bash
cargo run -p fret-demo
```

Notes:

- The demo requests a semantics snapshot every frame (`apps/fret-examples/src/components_gallery.rs`).
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
- Items include `pos_in_set` / `set_size` so AT can announce “Item X of Y” (ADR 0084).

### 2) Context menu

Trigger a context menu (where available in the demo) and:

1. Open the menu.
2. Navigate items via AT.
3. Invoke an item via AT “Click/Activate”.
4. Verify the menu closes and focus returns to the previous focus target.

Expected semantics outcomes:

- Context menu root is `Menu`.
- Items are exposed as `MenuItem` with correct `disabled/selected/expanded` flags.
- Items include `pos_in_set` / `set_size` so AT can announce “Item X of Y” (ADR 0084).

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
- List items include `pos_in_set` / `set_size` so AT can announce “Item X of Y” (ADR 0084).

## Recommended tooling (platform-specific)

### Windows

- Narrator (built-in) to validate navigation + invoke behavior.
- “Accessibility Insights for Windows” or “Inspect.exe” to inspect the UIA tree roles/flags.

### macOS

- VoiceOver (built-in).
- Accessibility Inspector to inspect the AX tree.

## Regression tests (keep these green)

Collection position metadata checks:

- cmdk list items:
  - `ecosystem/fret-ui-shadcn/src/command.rs` (`cmdk_arrow_moves_highlight_while_focus_stays_in_input`)
- Select popover items:
  - `ecosystem/fret-ui-shadcn/src/select.rs` (`select_popover_items_have_collection_position_metadata`)
- Context menu items (separator excluded from count):
  - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (`context_menu_items_have_collection_position_metadata_excluding_separators`)

Platform mapping check (AccessKit):

- `crates/fret-a11y-accesskit/src/lib.rs` (`list_item_pos_in_set_and_set_size_are_emitted`)

Run:

```bash
cargo nextest run -p fret-ui-shadcn
cargo nextest run -p fret-a11y-accesskit
```

## Known limitations / follow-ups

- Semantics schema is intentionally minimal (ADR 0033). It currently does not model combobox-to-list
  relationships (ARIA `controls/owns/activedescendant`-style linkage). If parity needs it, extend
  `fret-core` semantics schema behind an ADR + tests.
- Very large menus/lists can be expensive to expose in full to assistive technology.
  Prefer virtualized list surfaces for long collections, and follow ADR 0084’s rules:
  `active_descendant` must reference an item that is present in the current snapshot and within the
  active modal barrier scope, and visible items should include `pos_in_set` / `set_size` when known.
