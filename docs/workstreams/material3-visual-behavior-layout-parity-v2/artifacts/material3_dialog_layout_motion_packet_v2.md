# Material3 Dialog Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-077

## Truth

- Dialog panels use Compose Material3 `280dp..560dp` width bounds, remain centered in the
  viewport, and preserve a 24dp viewport margin before the panel clamps for small surfaces.
- AlertDialog content uses a 24dp panel padding slot, 16dp title-to-supporting spacing, 24dp
  supporting-to-actions spacing, and 8dp action button spacing.
- Dialog exposes stable automation anchors for `.scrim`, `.scrim.chrome`, `.panel`,
  `.panel.chrome`, `.headline`, `.supporting-text`, and `.actions`.
- The panel semantics is `Dialog` and is labelled/described by the rendered headline and
  supporting text nodes.
- Dialog enter/exit still uses the shared Material modal fade/rise/scale path, so the fixed-frame
  motion proof confirms the existing modal foundation remains wired.

## Sources

- Compose Material3 `AlertDialog.kt`: `BasicAlertDialog` constrains dialog width with
  `DialogMinWidth = 280.dp` and `DialogMaxWidth = 560.dp`.
- Compose Material3 `AlertDialog.kt`: AlertDialog content uses 24dp dialog padding, 16dp title
  bottom padding, 24dp text bottom padding, and `ButtonsMainAxisSpacing = 8.dp`.
- Compose Material3 `AlertDialog.kt`: the dialog content is given dialog pane semantics.
- Compose Material3 `DialogTokens.kt`: AlertDialog uses `SurfaceContainerHigh`, Level3
  elevation, `CornerExtraLarge`, headline typography, supporting text typography, and action label
  tokens.
- Base UI Dialog sources confirm the headless relation pattern: the dialog popup is labelled by
  Title and described by Description.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and
Base UI references were enough for the audited layout, semantics, and motion axes.

## Layer Finding

This packet found a Material recipe/proof-density gap, not a core or kit overlay policy gap:

- `fret-ui-kit` overlay policy already supplied modal overlay requests, focus trap/restore,
  dismissal wiring, and inert closing behavior. Existing Dialog focus/dismiss gates stayed green.
- `ecosystem/fret-ui-material3/src/foundation/modal_motion.rs` was already sufficient: the
  first-frame open/close test passed before the layout/a11y recipe repair.
- The Material Dialog recipe conflated viewport padding with panel padding, relied on a
  `Fill + max_width` shape that did not clamp the semantics panel to 560dp in the harness, and
  placed content directly in a container rather than an explicit vertical content column.
- The recipe also lacked stable headline/supporting/actions part ids and did not wire panel
  `labelled_by` / `described_by` relations to the rendered text parts.

## Artifacts

- `ecosystem/fret-ui-material3/src/dialog.rs`
- `ecosystem/fret-ui-material3/src/tokens/dialog.rs`
- `ecosystem/fret-ui-material3/tests/dialog_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `goldens/material3-headless/v1/material3-menu-dialog-style.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gates before the fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test dialog_state
```

The first red state failed because `m3-dialog.headline` did not exist. After adding the part ids
and relations, the next red state proved the old panel layout did not enforce the Material 560dp
max width in a 640px viewport. A later red state showed headline and supporting text bounds
overlapped because panel children were being attached directly to the container instead of a
vertical content column.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test dialog_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_dialog_and_bottom_sheet_expose_stable_part_test_ids
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_menu_dialog_style_suite_goldens_v1; Remove-Item Env:\FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_focus_is_contained_and_restored_across_schemes dialog_style_overrides_apply_to_container_and_text dialog_scrim_dismisses_without_activating_underlay material3_headless_menu_dialog_style_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --lib dialog
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

The refreshed headless goldens record the intentional centered 560dp panel and real vertical
content stack. Existing focus containment, focus restore, scrim dismissal, and style override gates
remained green.

## Residual Risk

- This packet proves the current AlertDialog-style API surface. Full-screen dialog variants,
  custom platform-specific compact pointer padding, predictive-back choreography, draggable
  resize, and nested-scroll/content-overflow behaviors are future API work.
- Dialog behavior remains `covered_v1` because the existing focus/dismiss tests stayed green, but
  a later overlay-family packet should still compare Dialog, Menu, DropdownMenu, Snackbar, and
  Tooltip as a group for shared policy drift.
