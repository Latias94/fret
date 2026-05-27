# Material 3 Overlay And Feedback Packet v1

Date: 2026-05-27
Task: M3CAS-070
Scope: `Menu`, `DropdownMenu`, `Dialog`, `BottomSheet`, `Tooltip`, and `Snackbar`

## Outcome Contract

This packet uses the Material source-alignment rule from the workstream: Material owns visual
outcomes and component-specific composition, `fret-ui-kit` owns reusable overlay policy, and
`crates/fret-ui` only changes when a mechanism contract gap is proven.

Observable outcomes for this packet:

- Menus and dropdown menus expose stable surface/item selectors while keeping roving focus,
  typeahead, Escape dismissal, outside-press dismissal, and focus restore in reusable menu policy.
- Dialogs expose stable scrim/panel selectors, use dialog semantics on the panel, and keep modal
  focus trap/restore in kit overlay policy.
- Bottom sheets expose stable scrim/sheet/drag-handle selectors without adding layout-sensitive
  chrome aliases that would change current sizing.
- Tooltips expose stable base and chrome selectors while preserving provider delay and safe-hover
  policy.
- Snackbars pass stable root selectors into the kit toast layer instead of duplicating toast
  viewport, action, close, and live-region behavior in Material recipes.

## Component Classification

| Component | Classification | Result |
| --- | --- | --- |
| `Menu` | `material_recipe`, `test_harness` | Added root `.chrome` and preserved item `.chrome` selectors. Roving focus/typeahead stay local to the recipe until another consumer proves a shared kit abstraction. |
| `DropdownMenu` | `kit_policy`, `material_recipe`, `test_harness` | Uses the shared dismissible menu path for Escape/outside dismissal and focus restore. The live menu surface inherits Material menu chrome selectors. The outside-press test was repaired because the previous underlay probe landed inside the open menu. |
| `Dialog` | `kit_policy`, `material_recipe`, `diagnostics` | Dialog now derives dotted `scrim`, `scrim.chrome`, `panel`, and `panel.chrome` selectors from the base id. The panel now reports `SemanticsRole::Dialog`; focus trap/restore remains kit overlay policy. |
| `BottomSheet` | `kit_policy`, `material_recipe`, `test_harness` | Modal bottom sheet derives dotted `scrim`, `scrim.chrome`, `sheet`, and `sheet.drag-handle` selectors; docked bottom sheet derives `drag-handle`. Root/sheet `.chrome` aliases were withheld because adding semantic markers to layout-critical containers changed the headless scene width. |
| `Tooltip` | `kit_policy`, `material_recipe`, `mechanism_follow_on` | Plain and rich tooltips now accept `test_id` and expose `.chrome`. Provider delay/safe-hover stay in kit policy. Rich tooltip interactivity remains a mechanism follow-on because tooltip overlays are currently click-through. |
| `Snackbar` | `kit_policy`, `material_recipe`, `test_harness` | `Snackbar::test_id` forwards to `ToastRequest::test_id`, keeping toast viewport/action/close/live-region details in `fret-ui-kit`. Subpart selectors can be added in kit if a consumer needs them. |

## Stable Selector Surface

New or confirmed selector contracts:

- `menu`
- `menu.chrome`
- `menu.item`
- `menu.item.chrome`
- `dropdown_menu`
- `dropdown_menu.chrome`
- `dropdown_menu.item`
- `dropdown_menu.item.chrome`
- `dialog.scrim`
- `dialog.scrim.chrome`
- `dialog.panel`
- `dialog.panel.chrome`
- `bottom_sheet.drag-handle`
- `modal_bottom_sheet.scrim`
- `modal_bottom_sheet.scrim.chrome`
- `modal_bottom_sheet.sheet`
- `modal_bottom_sheet.sheet.drag-handle`
- `tooltip`
- `tooltip.chrome`
- `snackbar`

Intentionally not added in this packet:

- `bottom_sheet.chrome`
- `modal_bottom_sheet.sheet.chrome`

Those aliases need a layout-safe semantic alias mechanism or an explicit component sizing decision.

## Implementation Anchors

- `ecosystem/fret-ui-material3/src/menu.rs`
- `ecosystem/fret-ui-material3/src/dropdown_menu.rs`
- `ecosystem/fret-ui-material3/src/dialog.rs`
- `ecosystem/fret-ui-material3/src/bottom_sheet.rs`
- `ecosystem/fret-ui-material3/src/tooltip.rs`
- `ecosystem/fret-ui-material3/src/snackbar.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-snackbar.*.json`
- `goldens/material3-headless/v1/material3-menu-dialog-style.*.json`
- `goldens/material3-headless/v1/material3-bottom-sheet.*.json`

## Gates

Passed on 2026-05-27:

```powershell
cargo fmt --package fret-ui-material3
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test radio_alignment snackbar_action_emits_command_and_dismisses
cargo nextest run -p fret-ui-material3 --test radio_alignment snackbar_dismiss_button_dismisses_without_emitting_command
cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_focus_is_contained_and_restored_across_schemes
cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_style_overrides_apply_to_container_and_text
cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_scrim_dismisses_without_activating_underlay
cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip_opens_and_closes_on_hover_across_schemes
cargo nextest run -p fret-ui-material3 --test radio_alignment rich_tooltip_opens_and_closes_on_hover_smoke
cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip_does_not_open_on_touch_move
cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes
cargo nextest run -p fret-ui-material3 --test radio_alignment dropdown_menu_dismisses_and_restores_focus_across_schemes
cargo nextest run -p fret-ui-material3 --test radio_alignment menu_pressed_scene_structure_is_stable
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_snackbar_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_menu_dialog_style_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_snackbar_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_menu_dialog_style_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1
```

## Follow-Ons

- `M3CAS-070-F1`: Add a layout-safe semantic alias/test-id marker for chrome selectors on
  layout-critical nodes, then revisit bottom sheet root/sheet `.chrome`.
- `M3CAS-070-F2`: Decide whether rich tooltip actions should be interactive. If yes, split an
  ADR-backed overlay mechanism change because current tooltip overlays are intentionally
  click-through.
- `M3CAS-070-F3`: Add kit-level toast action/close part selectors if Snackbar consumers need
  subpart automation.
- `M3CAS-070-F4`: De-duplicate plain/rich tooltip test-id/chrome wiring after selector behavior is
  locked.
- `M3CAS-070-F5`: Decide whether the docked bottom sheet headless scene should be content-sized or
  constrained by the gallery/test harness.
