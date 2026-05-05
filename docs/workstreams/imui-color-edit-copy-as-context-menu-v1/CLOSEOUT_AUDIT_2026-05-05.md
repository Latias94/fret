# ImUi Color Edit Copy-As Context Menu v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `ColorEditCopyOptions` as the per-control switch for editor `ColorEdit` copy-as behavior.
- Added a root swatch context menu opened by right-click, macOS Ctrl+click, Shift+F10, and the
  ContextMenu key.
- Added Dear ImGui-style copy payloads: float tuple, integer tuple, `#RRGGBB`, and `#RRGGBBAA`
  when alpha is visible.
- Routed copy activation through `Effect::ClipboardWriteText`.
- Kept the implementation in `fret-ui-editor`; no runtime, `fret-imui`, or global color-edit option
  state was added.

## Evidence

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/copy.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `repo-ref/imgui/imgui_widgets.cpp`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

## Gates Run

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-copy-as-context-menu-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/check_layering.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Eyedropper behavior.
- Picker options popup thumbnail previews.
- Higher-fidelity picker side-preview polish.
