# ImUi Debug Draw Response Surface v1 Evidence and Gates

Status: Closed.

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

## Gates

```bash
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
cargo nextest run -p fret-cookbook --lib cookbook_imui_debug_draw_example_keeps_current_facade_teaching_surface --no-fail-fast
cargo build -p fret-cookbook --example imui_debug_draw_basics --features cookbook-imui
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-debug-draw-basics/cookbook-imui-debug-draw-basics-smoke.json --json
FRET_DIAG=1 cargo run -p fretboard-dev -- diag suite cookbook-imui-debug-draw-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_debug_draw_basics
python tools/check_diag_scripts_registry.py
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python tools/check_layering.py
git diff --check
```
