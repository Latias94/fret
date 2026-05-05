# ImUi Color Edit History Swatches v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `ColorEditOptions::history` as an app-owned recent color source.
- Rendered non-empty history swatches before the palette row in the `ColorEdit` popup.
- Reused the same swatch row implementation for history and palette activation/drag behavior.
- Kept history recording, deduplication, capacity, and ordering policy app-owned.

## Evidence

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/swatches.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`

## Gates Run

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-history-swatches-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/check_layering.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Eyedropper behavior.
- Full picker preview and right-click context popup polish.
- Optional app-level history recording examples in cookbook/demo surfaces.
