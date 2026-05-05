# ImUi Debug Draw Response Surface v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `DebugDrawResponse`, carrying `ResponseExt`, `DebugDrawListSummary`, and command summaries.
- Added `DebugDrawInteractionOptions` so authors can opt a debug-draw canvas into pressable
  hover/click/drag/rect response queries.
- Kept the default helper paint-only to avoid changing event routing for overlays.
- Updated the cookbook proof to read summaries after the draw call and expose response metadata.
- Updated the promoted diagnostics smoke to wait for the response metadata anchor.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `tools/diag-scripts/cookbook/imui-debug-draw-basics/cookbook-imui-debug-draw-basics-smoke.json`
- Local launched run artifact (not checked in):
  `target/fret-diag/1778024517373-cookbook-imui-debug-draw-basics-smoke`

## Gates Run

```bash
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
cargo nextest run -p fret-cookbook --lib cookbook_imui_debug_draw_example_keeps_current_facade_teaching_surface --no-fail-fast
cargo build -p fret-cookbook --example imui_debug_draw_basics --features cookbook-imui
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-debug-draw-basics/cookbook-imui-debug-draw-basics-smoke.json --json
python tools/check_diag_scripts_registry.py
FRET_DIAG=1 cargo run -p fretboard-dev -- diag suite cookbook-imui-debug-draw-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_debug_draw_basics
```

## Residual Gaps

- This is canvas-level response only, not per-geometry hit testing.
- Renderer draw-call/scissor attribution remains deliberately separate.
- Pixel/layout diagnostics should stay in narrower diagnostics follow-ons.
