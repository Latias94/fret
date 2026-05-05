# ImUi Color Edit Side Preview Column v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Changed `ColorEdit` popup side previews from a horizontal row to a vertical current/original
  column.
- Added picker + side-preview row composition so the side preview sits beside the picker.
- Added a wider popup width only for picker + side-preview composition.
- Added explicit 3:2 side-preview swatch sizing.
- Kept current/original restore and alpha visibility behavior unchanged.
- Kept the implementation in `fret-ui-editor`; no runtime, platform, renderer, or global option
  state was added.

## Evidence

- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`

## Gates Run

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-side-preview-column-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/check_layering.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Platform-owned screen sampling.
- Higher-fidelity picker visual parity.
- Screenshot-backed gallery/diag evidence for the full `ColorEdit` popup.
