# Material3 Tooltip Layout Accessibility Motion Packet v2

Date: 2026-05-29
Task: M3PV2-079

## Truth

- Plain Tooltip uses the Compose Material3 40dp minimum width, 24dp minimum height, 200dp maximum
  width, 8dp horizontal padding, and 4dp vertical padding.
- Rich Tooltip uses the Compose Material3 40dp minimum width, 24dp minimum height, 320dp maximum
  width, 16dp horizontal padding, title-aware vertical padding, and Material rich container
  elevation/shape.
- Tooltip content exposes `Tooltip` semantics, the trigger is described by the mounted tooltip
  content while open, and the tooltip popup is an assertive live region.
- Tooltip open and close frames use Material fade plus scale from/toward 0.8 through the shared
  Material overlay-motion foundation.
- Existing hover delay, safe-hover, touch suppression, click-through overlay, and rich tooltip
  part-id behavior stays owned by the kit tooltip substrate and existing Material tests.

## Sources

- Compose Material3 `Tooltip.kt`: `PlainTooltip` applies `TooltipMinWidth = 40.dp`,
  `TooltipMinHeight = 24.dp`, `plainTooltipMaxWidth = 200.dp`, and
  `PlainTooltipContentPadding = 8.dp x 4.dp`.
- Compose Material3 `Tooltip.kt`: `RichTooltip` applies `TooltipMinWidth = 40.dp`,
  `TooltipMinHeight = 24.dp`, `richTooltipMaxWidth = 320.dp`, 16dp horizontal padding, title/text
  baseline spacing, action spacing, `RichTooltipTokens.ContainerElevation`, and rich shape/color
  tokens.
- Compose Material3 `Tooltip.kt`: Tooltip visibility uses `FastSpatial` scale from 0.8 to 1.0 and
  `FastEffects` alpha from 0 to 1 through a `graphicsLayer`.
- Compose Material3 internal `BasicTooltip.kt`: tooltip popups set assertive live-region semantics
  and a tooltip pane title.
- Base UI Tooltip was used as a supporting headless reference for popup/trigger part composition
  and trigger-to-popup description wiring.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and Base
UI references were sufficient for the audited layout, accessibility, and motion axes.

## Layer Finding

This packet found a Material recipe/token proof-density gap, not a core or kit mechanism gap:

- `fret-ui-kit` already owned tooltip delay groups, safe hover, touch suppression, overlay
  click-through behavior, trigger `described_by` wiring, and popper placement. Existing tooltip
  behavior gates stayed green.
- `crates/fret-ui` already exposed tooltip roles, live-region flags, trigger description
  relations, render transforms, and opacity scene operations. No core mechanism change was needed.
- Material Tooltip had the shared overlay-motion helper wired, but it lacked a focused v2 gate and
  still used a single 240px width cap for both plain and rich tooltips.
- Material Tooltip did not apply Compose's 40x24dp tooltip surface minimums, and the rich
  no-title/content path inherited the older 12dp vertical padding instead of the Material
  text-only 4dp vertical padding path.
- Fret still has no dedicated `paneTitle` semantics field. This packet uses the available
  assertive live-region and trigger description relation; a future core a11y packet can add a
  portable pane-title field if Dialog/Drawer/Tooltip need a distinct platform mapping.

## Artifacts

- `ecosystem/fret-ui-material3/src/tooltip.rs`
- `ecosystem/fret-ui-material3/src/tokens/tooltip.rs`
- `ecosystem/fret-ui-material3/tests/tooltip_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-overlays.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- `PlainTooltip::into_element(...)` resolves separate plain tooltip width/min-size token accessors
  and applies them to the visible `.chrome` container.
- `RichTooltip::into_element(...)` resolves separate rich tooltip width/min-size token accessors,
  applies title-aware padding to the visible `.chrome` container, and keeps existing rich
  title/supporting-text part IDs.
- `tooltip_content_root(...)` now marks mounted tooltip content as `Tooltip` plus assertive live
  region while preserving the kit-owned trigger `described_by` relation.
- `tooltip_policy_root(...)` continues to drive opacity and render-transform scale through
  `foundation::overlay_motion::drive_overlay_open_close_motion`.

## Proof

Red gate before the fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tooltip_state
```

Failed because plain tooltip chrome capped at the old 240px width and lacked the 40x24dp minimum
surface, while rich tooltip chrome also capped at the old 240px width instead of 320dp.

Green gates:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tooltip_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_overlays_suite_goldens_v1; Remove-Item Env:\FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_overlays_suite_goldens_v1
```

The focused `tooltip_state` gate now proves plain 200dp and rich 320dp width caps, 40x24dp minimum
surface sizing, tooltip role, assertive live region, trigger `described_by` relation, first-open
fade-scale, and first-close fade-scale. Refreshed overlays goldens record the intentional settled
plain/rich tooltip geometry shift across scale and theme variants.

## Residual Risk

- Tooltip style remains `covered_v1` because this packet did not re-audit every color,
  typography, shape, and elevation token beyond the already covered headless style suite.
- Tooltip behavior remains `covered_v1`: delay groups, safe-hover corridor, touch suppression,
  click-through overlays, and hover close behavior stayed covered by existing kit/Material tests.
- Rich Tooltip action rows remain residual because the current Fret `OverlayLayerKind::Tooltip`
  is explicitly non-hit-testable. Adding interactive rich tooltip actions would require a
  design-system-agnostic kit/core policy decision, not a recipe-only patch.
- Fret lacks a distinct pane-title semantics mechanism. The current packet records assertive live
  region semantics and trigger description relation; a broader a11y packet should decide whether
  to add a portable pane-title field.
