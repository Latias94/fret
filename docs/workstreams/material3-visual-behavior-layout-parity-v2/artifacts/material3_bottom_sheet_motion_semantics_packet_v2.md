# Material3 Bottom Sheet Motion And Semantics Packet v2

Date: 2026-05-29
Task: M3PV2-046

## Truth

- Modal bottom sheets slide between hidden and expanded anchors by the sheet surface's own height,
  not by the full viewport height.
- Modal bottom sheet default motion uses Material motion-scheme channels: `DefaultSpatial` for the
  sheet offset and `DefaultEffects` for scrim alpha. The sheet panel itself does not fade.
- Modal bottom sheet semantics are dialog-like, while the scrim and drag handle expose Material
  default accessible names: `Close sheet` and `Drag handle`.
- Existing stable part ids remain intact for automation: `<base>.scrim`, `<base>.scrim.chrome`,
  `<base>.sheet`, `<base>.sheet.chrome`, and `<base>.sheet.drag-handle`.

## Sources

- Compose Material3 `ModalBottomSheet.kt`: scrim alpha uses `DefaultEffects`, and the modal sheet
  is hosted in a dialog-like full-screen container.
- Compose Material3 `BottomSheet.kt`: sheet show motion uses `defaultSpatialSpec`, sheet anchors
  place `Hidden` at full height and `Expanded` at `fullHeight - sheetHeight`, and the sheet surface
  has bottom-sheet pane semantics.
- Compose Material3 `SheetDefaults.kt`: drag handle size/padding/color and `Drag handle`
  content description.
- Compose Material3 English string table: `Close sheet`, `Bottom sheet`, and `Drag handle`.
- Fret shadcn Sheet remains the local layering exemplar for dialog semantics and modal overlay
  focus containment, not the Material visual truth.

## Artifacts

- `ecosystem/fret-ui-material3/src/bottom_sheet.rs`
- `ecosystem/fret-ui-material3/tests/bottom_sheet_motion.rs`
- `goldens/material3-headless/v1/material3-bottom-sheet.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- `ModalBottomSheet::new(...)` defaults now route through a component-local Material spring driver.
  Explicit `open_duration_ms` / `close_duration_ms` / `easing_key` remains as an override path for
  deterministic tests and intentional legacy timing overrides.
- The sheet surface is translated with `FractionalRenderTransformProps` on the surface-height
  wrapper, while an outer `InteractivityGate` keeps closing content inert.
- `DockedBottomSheetVariant::Modal` now exposes `SemanticsRole::Dialog` and `Bottom sheet`.
  The scrim pressable exposes `Close sheet`; the drag handle exposes `Drag handle`.

## Proof

Red gates before the fix:

```powershell
cargo nextest run -p fret-ui-material3 --test bottom_sheet_motion
```

Failed because the modal sheet first-open transform was `ty=500.95712` in a `520px` viewport, and
the modal sheet role was `Group` rather than `Dialog`.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test bottom_sheet_motion
cargo nextest run -p fret-ui-material3 --features diagnostics material3_dialog_and_bottom_sheet_expose_stable_part_test_ids --test automation_surface
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1
```

Result: the fixed-frame test proves own-height slide and no panel fade; the semantics test proves
dialog/scrim/drag-handle names; diagnostics automation part ids stayed green; headless bottom sheet
goldens were refreshed because settled modal output no longer carries a redundant transform
wrapper.

## Layer Decision

This is a Material recipe/foundation-adjacent issue, not a `crates/fret-ui` mechanism gap:

- `FractionalRenderTransformProps`, `InteractivityGateProps`, overlay requests, and focus trap
  primitives already existed.
- The drift was in how the Material bottom sheet recipe chose anchors, motion channels, and
  semantics names.
- No design-system-agnostic policy was pushed into `fret-ui-kit`.

## Residual Risk

- Drag gestures, partial expansion, predictive-back scaling, and nested-scroll damping are still
  not implemented for Fret Material3 bottom sheets.
- Bottom sheet behavior remains `covered_v1` because dismissal/focus containment already had
  existing coverage, but a future M3PV2-050 overlay packet should still compare all overlay
  families together for shared policy drift.
