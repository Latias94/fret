# Material 3 Visual Behavior Layout Parity v2 - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

The lane is open. M3PV2-010 is complete: the v2 parity-axis matrix exists and covers all 39
components from the closed Material3 component sweep. M3PV2-021 is complete: Material3 Select now
uses dotted `<base>.listbox` ids for the listbox automation surface. M3PV2-022 is complete:
Autocomplete fallback ids now use the same dotted listbox contract, ExposedDropdown proves
combobox/listbox wiring through composition, and live Material3 Select diagnostics have been swept
to the dotted ids. M3PV2-023 is complete: TextInput/TextArea gained labelled/described relation
targets, Material TextField wires visual label/supporting text into those relations, and the filled
chrome test now tracks the current container + active-indicator layer split.
M3PV2-024 is complete: the Material field text-start inset helper moved from Select into
`foundation::field`, TextField uses it for leading-icon input padding, floating label, and
supporting text, and fixed geometry gates now cover idle/focus/populated label positions.
M3PV2-025 is complete: Autocomplete and ExposedDropdown popup geometry now anchors to the
TextField chrome element when available, so icon-bearing fields get menu/listbox width parity with
the full field while the input remains the combobox trigger and keyboard/a11y owner.
M3PV2-026 is complete: Select selected menu items now use selected content colors for label,
leading icon, and trailing icon, and the visible item chrome uses the Material selectable-item inset
inside the listbox while the pressable row keeps the existing behavior contract.

## Decisions

- This lane is about shadcn-level proof density, not shadcn visual styling.
- Material spec, Compose Material3, MUI Material UI, and Base UI are axis-specific references.
- Stable Fret-side shadcn components are exemplars for layering and gates only.
- Layout defaults must be classified as intrinsic recipe defaults or caller-owned before edits.
- Shared foundation refactors require multiple component proofs.

## Next Recommended Action

Continue M3PV2-020 with another field-family packet. Good next candidates are a dedicated
multiline TextField scenario, true popup surface style/elevation proof for Autocomplete/
ExposedDropdown, or fixed-timestep popup/field motion. Do not mark motion axes complete from
settled-geometry evidence; TextField, Select, and popup motion still need transition packets.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
```
