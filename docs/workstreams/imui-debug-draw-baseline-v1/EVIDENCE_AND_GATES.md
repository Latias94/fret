# ImUi Debug Draw Baseline v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `crates/fret-ui/src/canvas.rs`: the runtime already exposes a declarative canvas paint surface.
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`: the IMUI gap audit tracks the debug-draw gap
  against the Dear ImGui reference.

## Implementation Anchors

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`

## Gates

```bash
cargo fmt --package fret-ui-kit
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-baseline-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
