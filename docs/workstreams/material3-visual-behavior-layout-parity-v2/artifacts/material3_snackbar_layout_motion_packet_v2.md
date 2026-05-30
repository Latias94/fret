# Material3 Snackbar Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-078

## Truth

- SnackbarHost applies the Compose Material3 12dp host inset and keeps the visible snackbar
  centered in the bottom-center host lane by default.
- Snackbar containers cap at 600dp wide and keep Material minimum heights: 48dp for single-line
  content and 68dp for two-line content.
- Snackbar exposes a polite live region labelled `Alert`, and the close affordance is labelled
  `Dismiss`.
- Snackbar enter and close frames use Material fade plus scale from/toward 0.8, without the generic
  Sonner Y-slide transform.
- Existing action, close, timer, and removal behavior remains owned by the kit toast substrate and
  stays green for the Material recipe.

## Sources

- Compose Material3 `Snackbar.kt`: `Snackbar(snackbarData, ...)` applies 12dp host padding,
  `ContainerMaxWidth = 600.dp`, `SnackbarVerticalPadding = 14.dp`, and 48dp/68dp single-line and
  two-line container heights.
- Compose Material3 `SnackbarHost.kt`: visible snackbars use `FadeInFadeOutWithScale`, scaling
  from 0.8 to 1.0 while opacity animates, and the visible item exposes polite live-region
  semantics.
- Compose Material3 English string table: `SnackbarPaneTitle` maps to `Alert`, and
  `SnackbarDismiss` maps to `Dismiss`.
- Base UI Toast was used as a supporting headless reference for toast-root/viewport accessibility
  shape; Compose remains the visual and motion truth for this packet.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and Base
UI references were sufficient for the audited layout, accessibility, and motion axes.

## Layer Finding

This packet found a Material recipe gap plus a reusable kit toast-surface gap, not a core
mechanism gap:

- `fret-ui-kit` already owned toast persistence, queueing, actions, close buttons, timers, live
  regions, viewport focus, and swipe behavior. Existing kit toast tests stayed green.
- `fret-ui-kit` `ToastLayerStyle` only had the Sonner-like Y-slide path. It needed a
  design-system-agnostic optional `scale_from` style slot so Material could express fade-scale
  snackbar motion while shadcn/Sonner defaults remain unchanged.
- The kit renderer accepted single-line/two-line min-height style tokens but used them only to
  estimate stack height. The visible toast surface now applies those min-height tokens to layout
  too.
- Material `SnackbarHost` still inherited generic kit defaults: 356px width, 24px desktop offset,
  16px mobile offset, and generic close labelling. The recipe now wires Material 12dp host inset,
  600dp max width, `Alert` live-region label, `Dismiss` close label, and zero-slide fade-scale
  motion.
- No `crates/fret-ui` mechanism change was required: semantics live-region support, render
  transforms, hit-testing, and layout inspection already existed.

## Artifacts

- `ecosystem/fret-ui-kit/src/window_overlays/requests.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- `ecosystem/fret-ui-material3/src/snackbar.rs`
- `ecosystem/fret-ui-material3/src/tokens/snackbar.rs`
- `ecosystem/fret-ui-material3/tests/snackbar_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `goldens/material3-headless/v1/material3-snackbar.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- `SnackbarHost::into_element(...)` requests a toast layer with Material defaults: bottom-center
  position, one visible snackbar, 12dp margin and mobile offset, 600dp max width unless the caller
  supplies an explicit width override, and `Alert` as the viewport/live-region label.
- `snackbar_toast_layer_style(...)` maps Material token colors, typography, padding, height, and
  duration into the shared kit `ToastLayerStyle`, sets `slide_distance` to zero, and sets
  `scale_from` to 0.8.
- The kit toast renderer applies `scale_from` through the existing presence opacity progress and
  applies the style min-height to the real pressable/container layout, not only to stack
  estimation.

## Proof

Red gate before the fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test snackbar_state
```

Failed because the settled Snackbar measured at the generic 356px kit toast width instead of the
Material 600dp width cap, and the first open frame had no Material scale transform.

Green gates:

```powershell
cargo fmt --package fret-ui-kit --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test snackbar_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_snackbar_suite_goldens_v1; Remove-Item Env:\FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment snackbar
cargo nextest run -p fret-ui-kit --lib toast
cargo check -p fret-ui-kit --tests
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-kit --tests --no-deps -- -D warnings
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

The focused `snackbar_state` gate now proves 600dp centered width, 48/68dp minimum surface height,
`Alert` live-region labelling, `Dismiss` close labelling, first-open fade-scale, and first-close
fade-scale without a Sonner Y-slide. Refreshed headless goldens record the intentional settled
600dp/48dp/68dp geometry.

## Residual Risk

- Snackbar style remains `covered_v1` because this packet did not re-audit every color,
  typography, elevation, and action-button state token beyond the already covered headless style
  suite.
- Snackbar behavior remains `covered_v1` because queueing, action dispatch, close dispatch,
  timers, and removal behavior stayed covered by existing kit/Material tests. A later overlay
  family packet can still compare Snackbar, Tooltip, Menu, DropdownMenu, Dialog, and BottomSheet
  policy drift as a group.
- Multi-snackbar stacking and swipe gesture parity remain kit-level residual work rather than a
  current Material recipe gap.
