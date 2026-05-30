# Material 3 Segmented Button Diagnostics Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Closed the SegmentedButtonSet matrix residual with a promoted diagnostics suite and focused
semantics/golden gates.

## Result

- Material3 segmented-button roving-semantics diagnostics passed.
- Segmented-button semantics roles and headless golden Rust gates passed.
- No component, foundation, kit-policy, or mechanism change was needed.

## Gates

- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json`
- `python -m json.tool tools/diag-scripts/suites/ui-gallery-material3-segmented-button-roving-semantics-screenshots/suite.json`
- `python tools/check_diag_scripts_registry.py`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json --dir target/fret-diag/material3-segmented-button-roving-semantics-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment segmented_button_semantics_roles_match_compose_baseline`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_segmented_button_suite_goldens_v1`

## Layering

- `material_recipe`: single/multi selection, item roles, checked state, per-segment chrome, and
  selectors.
- `material_foundation`: shared indication state-layer/ripple and minimum interactive target sizing.
- `diagnostics`: promoted the Material3 gallery roving-semantics gate and recorded bundle evidence.
- `kit_policy`: no new shared policy gap was found.
- `mechanism`: no core mechanism gap was found.
