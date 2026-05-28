# Material 3 Menu And Dropdown Diagnostics Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Closed the Menu and DropdownMenu matrix residuals with dedicated Material3 diagnostics evidence and
focused Rust gates.

## Result

- Material3 Menu focus/dismiss diagnostics passed.
- Existing Material3 menu item chrome diagnostics passed.
- Automation-surface, DropdownMenu dismiss/restore, Menu pressed-scene, and Menu style override
  Rust gates passed.
- No component, foundation, kit-policy, or mechanism change was needed.

## Gates

- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json`
- `python tools/check_diag_scripts_registry.py`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json --dir target/fret-diag/material3-menu-focus-dismiss-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-item-chrome-fill.json --dir target/fret-diag/material3-menu-item-chrome-fill-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_menu_and_dropdown_expose_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment dropdown_menu_dismisses_and_restores_focus_across_schemes`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment menu_pressed_scene_structure_is_stable`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment menu_style_overrides_apply_to_container_and_label`

## Layering

- `material_recipe`: Menu surface/item chrome, selectors, item state, and static Menu composition.
- `kit_policy`: DropdownMenu Escape/outside-press dismissal, non-click-through behavior, overlay
  unmount, and focus restore.
- `diagnostics`: added the missing Material3 focus/dismiss gallery gate and reused chrome-fill
  diagnostics.
- `material_foundation`: no new shared foundation gap was found.
- `mechanism`: no core mechanism gap was found.
