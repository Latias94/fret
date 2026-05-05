# ImUi Color Edit Alpha Preview Options v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui exposes alpha preview policy through `ColorButton` / `ColorEdit` flags:
`AlphaOpaque`, `AlphaNoBg`, and `AlphaPreviewHalf`, with transparent checkerboard preview as the
default. Fret now exposes the same product choice as explicit per-control editor policy instead of
global `SetColorEditOptions()` state.

## Ownership

- `color_edit.rs` owns the public `ColorEditAlphaPreview` option.
- `popup/preview.rs` owns checkerboard, opaque, no-background, and half-alpha preview rendering.
- `popup/swatches.rs` applies the selected preview mode to preset swatches.
- The main swatch applies the same preview mode through `ColorEditOptions`.

## Must-Be-True Outcomes

- Existing controls default to checkerboard-backed transparent preview.
- `Opaque` forces preview alpha to `1.0` without changing the stored color.
- `NoBackground` renders the actual alpha without checkerboard background.
- `Half` renders an opaque half next to a checkerboard-backed transparent half.

## Non-Goals

- No global color edit options state.
- No color payload drag/drop.
- No HueWheel picker.
- No palette customization or history.
