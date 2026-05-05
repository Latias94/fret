# ImUi Debug Draw Diag Smoke v1 Evidence and Gates

Status: Closed.

## Evidence

- `tools/diag-scripts/cookbook/imui-debug-draw-basics/cookbook-imui-debug-draw-basics-smoke.json`
- `tools/diag-scripts/suites/cookbook-imui-debug-draw-basics/suite.json`
- `tools/diag-scripts/index.json`
- `apps/fret-cookbook/EXAMPLES.md`
- `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
- Local launched run artifact (not checked in):
  `target/fret-diag/1777994933876-cookbook-imui-debug-draw-basics-smoke`

## Gates

```bash
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-debug-draw-basics/cookbook-imui-debug-draw-basics-smoke.json --json
python tools/check_diag_scripts_registry.py
FRET_DIAG=1 cargo run -p fretboard-dev -- diag suite cookbook-imui-debug-draw-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_debug_draw_basics
git diff --check
```
