# Material 3 Chip Visual Diagnostics Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Closed visual follow-ons for AssistChip, SuggestionChip, FilterChip, and InputChip.

## Result

- Added a promoted diagnostics script for representative State Matrix chip visual chrome.
- The script passed without component changes.
- Focused automation-surface and semantics/roving Rust gates passed.
- No foundation, kit-policy, or mechanism gap was found.

## Gates

- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json --dir target/fret-diag/material3-chip-visual-chrome-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment chips_export_checked_state_for_selected_semantics chip_set_roving_treats_trailing_action_focus_as_active_chip`

## Layering

- `material_recipe`: chip composition, selected semantics, variant chrome, and trailing actions.
- `material_foundation`: shared indication/ripple and minimum target sizing.
- `diagnostics`: new gallery script.
- `kit_policy`: no new shared policy was found.
- `mechanism`: no core mechanism gap was found.
