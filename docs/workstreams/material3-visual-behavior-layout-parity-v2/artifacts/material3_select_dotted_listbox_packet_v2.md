# Material 3 Select Dotted Listbox Packet v2

Date: 2026-05-28
Status: Done

## Truth

- Material3 Select listbox ids should follow the current field-family part-id convention:
  `<base>.listbox`.
- A Select without a caller-provided base id should use `material3-select.listbox`.
- Diagnostics and tests should not keep the obsolete `<base>-listbox` convention alive for current
  Material3 Select work.
- The change is a recipe-level automation-surface repair. It does not require a core mechanism,
  shared kit policy, or Material foundation refactor.

## Sources

- Current field-family selector convention:
  `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_behavior_packet_v1.md`
- v2 parity axis matrix:
  `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`
- Fret-side exemplar for stable Select automation surfaces:
  `docs/audits/shadcn-select.md`
- Base UI Select parts as headless reference:
  `repo-ref/base-ui/packages/react/src/select/`

## Refactor

- `ecosystem/fret-ui-material3/src/select.rs`
  - Default listbox id changed from `material3-select-listbox` to `material3-select.listbox`.
  - Base-derived listbox id now uses `foundation::test_id::part_test_id(base, "listbox")`.
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`
  - Focused Select behavior tests now assert `select-trigger.listbox`.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
  - Automation surface test now asserts `m3-select.listbox`.
- `tools/diag-scripts/ui-gallery/{overlay,resizable}/`
  - Material3 Select scripts now target dotted `.listbox` ids.

## Proof

```powershell
cargo fmt --package fret-ui-material3
python -m json.tool tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-multi-viewport-select-placement.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-dialog-a11y-parity-bundle.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-dialog-overlay-screenshots.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-dialog-a11y-bundle.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-overlay-parity-screenshots.json | Out-Null
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --lib select
```

Results:

- `select_behavior`: 8 passed, run ID `2501e112-bd78-4998-ab53-23cd76a6dc1e`.
- `automation_surface material3_select_exposes_stable_part_test_ids`: 1 passed, run ID
  `221bfc02-8850-4173-8aa6-6ef8d802f994`.
- `--lib select`: 21 passed, run ID `113bd029-2012-46bf-ab00-6b4c06816028`.

## Residual Risk

- This packet closes the first Select v2 automation-surface gap. It does not claim full Select
  visual pixel parity, motion parity, or complete Material token parity.
- Historical v1 artifacts may still mention `<base>-listbox`; those remain historical evidence and
  should not be used as the current v2 selector contract.
