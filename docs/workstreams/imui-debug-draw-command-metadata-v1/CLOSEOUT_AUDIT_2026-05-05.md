# ImUi Debug Draw Command Metadata v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added public `DebugDrawCommandKind`, `DebugDrawCommandSummary`, and `DebugDrawListSummary`.
- Added `ImUiDebugDrawList::command_summaries()` and `ImUiDebugDrawList::list_summary()`.
- Preserved active `ChannelsSplit` order in command summaries without mutating the draw list.
- Reported optional `ImageId` and payload counts for point, vertex, index, and triangle data.
- Kept the parity step bounded: no user render callbacks, raw draw buffers, or backend draw-call
  contract.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`

## Gates Run

```bash
cargo fmt --package fret-ui-kit -- --check
cargo nextest run -p fret-ui-kit --features imui debug_draw_list_reports_command_summaries_in_merge_order --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_list_summary_counts_visible_command_classes --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_options_default_to_clipped_canvas --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
python tools/check_layering.py
python tools/report_largest_files.py --top 30 --min-lines 800
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-command-metadata-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-debug-draw-triangle-mesh-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Command metadata is source-level IMUI metadata, not backend draw-call attribution.
- Callback/user draw commands remain intentionally out of the generic scene contract.
- Hit-test-aware debug draw interaction remains a separate editor tooling concern.
