# Material 3 Dialog Diagnostics Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Closed the Dialog matrix residual by adding dedicated Material3 Dialog diagnostics evidence for the
gallery modal path.

## Result

- Dedicated Dialog diagnostics passed against the Material3 UI Gallery page.
- The diagnostics bundle includes open-state panel, panel chrome, scrim, scrim chrome, action, and
  select selectors.
- Automation-surface, focus containment/restore, scrim dismiss, and style override Rust gates
  passed.
- No component, foundation, kit-policy, or mechanism change was needed.

## Gates

- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json`
- `python tools/check_diag_scripts_registry.py`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json --dir target/fret-diag/material3-dialog-focus-trap-restore-20260528-final --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_dialog_and_bottom_sheet_expose_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_focus_is_contained_and_restored_across_schemes`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_scrim_dismisses_without_activating_underlay`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_style_overrides_apply_to_container_and_text`

## Layering

- `material_recipe`: Dialog scrim/panel/action selectors, panel semantics role, and visual tokens.
- `kit_policy`: modal overlay request, barrier roots, Escape dismissal, focus containment, and focus
  restore.
- `diagnostics`: added the missing Material3-specific gallery script and promoted suite entry.
- `material_foundation`: no new shared foundation gap was found.
- `mechanism`: no core mechanism gap was found.
