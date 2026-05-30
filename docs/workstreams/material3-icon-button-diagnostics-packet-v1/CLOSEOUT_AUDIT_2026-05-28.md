# Material 3 IconButton Diagnostics Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Closed the IconButton matrix residual and repaired stale diagnostics navigation.

## Result

- Centered-chrome diagnostics passed after navigating to the dedicated Icon Button page.
- Automation-surface and pressed-scene Rust gates passed.
- No component, foundation, kit-policy, or mechanism change was needed.

## Gates

- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-icon-button-centered-chrome.json`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-icon-button-centered-chrome.json --dir target/fret-diag/material3-icon-button-centered-chrome-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment icon_button_pressed_scene_structure_is_stable`

## Layering

- `material_recipe`: variants, toggle semantics, shape morphing, and selectors.
- `material_foundation`: shared indication/ripple and minimum target sizing.
- `diagnostics`: stale page navigation repaired.
- `kit_policy`: no new shared policy was found.
- `mechanism`: no core mechanism gap was found.
