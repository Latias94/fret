# ImUi Debug Draw Concave Poly Fill v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the Dear ImGui `AddConcavePolyFilled` / `PathFillConcave` semantics follow-on above
the canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `ImUiDebugDrawList::add_concave_poly_filled`.
- Added `ImUiDebugDrawPath::fill_concave`.
- Added a dedicated concave polygon fill command variant.
- Lowered concave fill through the existing closed Canvas fill path.
- Kept the implementation in `fret-ui-kit::imui` without adding an IMUI triangulator, renderer
  tessellation policy, retained path state, or hit-testing contract.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for channel
splitting, hit-test-aware debug interaction, reusable draw-list command metadata, image loading
recipes, image tinting, arbitrary image quads, or multi-color rect fill. Rounded image clipping is
covered by `docs/workstreams/imui-debug-draw-rounded-image-v1/`.
