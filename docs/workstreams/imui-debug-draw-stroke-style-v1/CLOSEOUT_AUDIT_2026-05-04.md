# ImUi Debug Draw Stroke Style v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the first stroke-style depth slice for the canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `DebugDrawStrokeStyle` with width, join, cap, miter limit, and dash policy.
- Re-exported `DebugDrawStrokeStyle` from `fret-ui-kit::imui`.
- Preserved the old thickness-based calls as convenience wrappers.
- Added styled variants for line, polyline, rect, triangle, and circle commands.
- Kept width-only default lowering on `PathStyle::Stroke`; explicit policy lowers to
  `PathStyle::StrokeV2`.
- Added validation for invalid dash and miter inputs.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast`
  passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Clip rect stack support is covered by
`docs/workstreams/imui-debug-draw-clip-stack-v1/CLOSEOUT_AUDIT_2026-05-04.md`; image overlays are
covered by `docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md`. Start
separate follow-ons for channel splitting, hit-test-aware debug interaction, reusable draw-list
command metadata, or image loading recipes.
