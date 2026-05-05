# ImUi Debug Draw Clip Stack v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `docs/workstreams/imui-debug-draw-stroke-style-v1/CLOSEOUT_AUDIT_2026-05-04.md`: stroke style
  depth was closed before clipping stack depth.
- `docs/adr/0002-display-list.md`: scene clip operations are existing display-list mechanisms.

## Implementation Anchors

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`

## Gates

```bash
cargo fmt --package fret-ui-kit
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-clip-stack-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
