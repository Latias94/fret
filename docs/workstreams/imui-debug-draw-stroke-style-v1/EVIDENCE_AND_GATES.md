# ImUi Debug Draw Stroke Style v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `docs/workstreams/imui-debug-draw-shape-primitives-v1/CLOSEOUT_AUDIT_2026-05-04.md`: shape
  primitives were closed before stroke policy depth.
- `docs/adr/0080-vector-path-contract.md`: vector path style semantics are the existing contract.

## Implementation Anchors

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`

## Gates

```bash
cargo fmt --package fret-ui-kit
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-stroke-style-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
