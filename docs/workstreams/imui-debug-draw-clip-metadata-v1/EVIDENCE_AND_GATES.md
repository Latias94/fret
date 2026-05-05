# ImUi Debug Draw Clip Metadata v1 Evidence and Gates

Status: Closed.

## Evidence

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

## Gates

```bash
cargo fmt --package fret-ui-kit -- --check
cargo nextest run -p fret-ui-kit --features imui debug_draw_command_summaries_track_effective_clip_stack --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_list_reports_command_summaries_in_merge_order --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_options_default_to_clipped_canvas --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
python tools/check_layering.py
python tools/report_largest_files.py --top 30 --min-lines 800
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-clip-metadata-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
