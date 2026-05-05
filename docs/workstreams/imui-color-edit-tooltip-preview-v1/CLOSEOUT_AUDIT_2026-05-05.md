# ImUi Color Edit Tooltip Preview v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `ColorEditTooltipOptions` as the per-control equivalent of Dear ImGui's `NoTooltip` flag.
- Added a root swatch hover tooltip overlay for editor `ColorEdit`.
- Added a compact `ColorTooltip()`-style payload: hex, RGB, and HSV lines, with alpha following
  `show_alpha`.
- Reused the existing `ColorEditAlphaPreview` stack for tooltip preview rendering.
- Kept tooltip policy in `fret-ui-editor`; no runtime, `fret-imui`, or global color-edit option
  state was added.

## Evidence

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/tooltip.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `repo-ref/imgui/imgui_widgets.cpp`

## Gates Run

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-tooltip-preview-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/check_layering.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Eyedropper behavior.
- Copy-as/context options menu polish.
- Picker options popup thumbnail previews.
