# ImUi Item Behavior Kernel v1 TODO

Status: closed execution checklist
Last updated: 2026-04-24

## P0 - Freeze The Target

- [x] Audit the current item-like behavior paths in:
  - `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- [x] Name the minimum private kernel inputs and outputs before code motion.
- [x] Identify stale duplicate helpers that should be deleted after migration.
- [x] Confirm the first slice does not require `fret-ui`, runtime, or `Response` contract widening.

## P1 - Land The First Kernel Slice

- [x] Add the private item-behavior owner inside `ecosystem/fret-ui-kit/src/imui`.
- [x] Migrate button-like controls first.
- [x] Delete replaced button-family behavior instead of keeping a compatibility path.
- [x] Run focused `fret-ui-kit` and `fret-imui` gates.
- [x] Record the first-slice decision in a dated note only if it changes scope or contracts.

## P2 - Prove Additional Families

- [x] Migrate one second family, preferably boolean/selectable before overlay triggers.
- [x] Keep family visual/layout policy local while sharing the behavior kernel.
- [x] Add or adjust focused tests only around observable behavior.
- [x] Re-run showcase/editor proof builds.
- [x] Migrate selectable rows with pointer-click modifier reporting kept as an explicit kernel
  option.
- [x] Migrate combo triggers while keeping popup toggle/open policy local to combo controls.

## P3 - Decide Whether To Continue Or Split

- [x] If menu/tab trigger behavior is only shared item behavior, migrate it here; otherwise classify
  it out of this full pressable item kernel.
- [x] If menu/tab work becomes dismissal, focus trap/restore, submenu, or roving-depth policy, open
  a separate follow-on.
- [x] If text input, table, docking, debug draw, or public API widening becomes the driver, stop and
  create a narrower workstream.
- [x] Close this lane once the private kernel is proven and the duplicated paths are gone.
