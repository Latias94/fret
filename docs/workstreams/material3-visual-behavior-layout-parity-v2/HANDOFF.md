# Material 3 Visual Behavior Layout Parity v2 - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

The lane is open. M3PV2-010 is complete: the v2 parity-axis matrix exists and covers all 39
components from the closed Material3 component sweep. M3PV2-021 is complete: Material3 Select now
uses dotted `<base>.listbox` ids for the listbox automation surface.

## Decisions

- This lane is about shadcn-level proof density, not shadcn visual styling.
- Material spec, Compose Material3, MUI Material UI, and Base UI are axis-specific references.
- Stable Fret-side shadcn components are exemplars for layering and gates only.
- Layout defaults must be classified as intrinsic recipe defaults or caller-owned before edits.
- Shared foundation refactors require multiple component proofs.

## Next Recommended Action

Continue M3PV2-020. The first Select automation-surface slice is done; next pick either Select
visual/layout token proof or Autocomplete/ExposedDropdown popup choreography. Write the proof note
before editing code.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
```
