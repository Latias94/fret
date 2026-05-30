# Material3 TextField Floating Label Geometry Packet v2

Date: 2026-05-28
Task: M3PV2-024
Status: Done

## Truth

- Material field text, floating label, and supporting text share the same leading text start when a
  leading icon is present.
- The shared leading-icon text start is owned by Material field foundation, not hidden inside one
  recipe.
- TextField floating label rests at the expanded position when idle and empty, then settles to the
  minimized position when focused or populated.
- Field-family popup state can request the floating label/placeholder state through the existing
  `expanded` channel.

## Sources

- Compose Material3 `TextField.kt` places label, text field, placeholder, leading icon, and
  supporting text as one field layout with label progress:
  `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt`.
- Existing Fret Material Select already used the Material field leading-icon text-start rule.
- Fret field-family checklist:
  `.agents/skills/fret-material-source-alignment/references/material-field-family-checklist.md`.

## Layering

- Material foundation fix: `ecosystem/fret-ui-material3/src/foundation/field.rs`.
  - `material_field_text_start_inset_x` is now shared by field recipes.
- Material recipe fixes:
  - `TextField` uses the shared inset for input padding, label position, and supporting text.
  - `Select` reuses the foundation helper instead of owning a duplicate helper.
  - `TextField` treats `expanded` like a floating-label state input, matching popup field
    choreography.
- No `fret-ui` or `fret-ui-kit` mechanism/policy change was required.

## Artifacts

- `ecosystem/fret-ui-material3/src/foundation/field.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/text_field_hover.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_leading_icon_offsets_label_and_supporting_text
cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_geometry_tracks_idle_focus_and_populated_states
cargo nextest run -p fret-ui-material3 --test text_field_hover
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids material3_select_exposes_stable_part_test_ids
```

Results:

- `text_field_leading_icon_offsets_label_and_supporting_text`: passed, run id
  `dc6f6e7e-5f08-46dd-b938-2450ba82bb6a`.
- `text_field_floating_label_geometry_tracks_idle_focus_and_populated_states`: passed, run id
  `93644738-6595-4c61-bfdb-751801fd5556`.
- Full `text_field_hover`: 8 passed, run id `b3659aa1-eb02-4d2e-8358-8fec050b03ef`.
- `select_behavior`: 8 passed, run id `316a22f1-14e7-44bd-9f53-9bdb7ad8cf69`.
- TextField + Select automation-surface smoke: 2 passed, run id
  `fabaeeb3-307e-4200-99aa-5d176f282380`.

## Residual Risk

- This packet proves settled geometry, not the full per-frame floating-label animation curve.
- Multiline TextField still needs a dedicated Material scenario.
- Popup field width/chrome for Autocomplete and ExposedDropdown remains in M3PV2-020.
