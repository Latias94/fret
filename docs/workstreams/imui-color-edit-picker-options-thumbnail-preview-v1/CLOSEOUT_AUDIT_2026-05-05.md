# ImUi Color Edit Picker Options Thumbnail Preview v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Replaced text-only HueBar/HueWheel picker option controls with thumbnail radio cards.
- Reused existing SV picker, HueBar, and HueWheel preview helpers from `popup/picker.rs`.
- Kept popup-local runtime picker selection and AlphaBar toggle behavior unchanged.
- Kept the implementation in `fret-ui-editor`; no runtime, `fret-imui`, renderer, or global option
  state was added.

## Evidence

- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/options.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `repo-ref/imgui/imgui_widgets.cpp`

## Gates Run

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/check_layering.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Eyedropper behavior.
- Deeper side-preview polish.
- Full right-click context-menu parity for picker options.
