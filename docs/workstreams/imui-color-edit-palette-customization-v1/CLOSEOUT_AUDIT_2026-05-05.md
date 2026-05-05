# ImUi Color Edit Palette Customization v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `ColorEditPaletteEntry` and `default_color_edit_palette()` as the editor-owned palette
  source API.
- Added `ColorEditOptions::palette` so apps can provide their own palette entries while retaining
  the current 12-entry default palette.
- Routed palette entries through the `ColorEdit` popup into `popup/swatches.rs`.
- Preserved alpha on palette activation, matching the Dear ImGui custom palette demo behavior.
- Treated empty palettes as no preset row, so an empty custom palette does not keep a no-op popup
  alive by itself.

## Evidence

- `repo-ref/imgui/imgui_demo.cpp`
- `ecosystem/fret-ui-editor/src/controls/mod.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/swatches.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`

## Gates Run

```bash
cargo fmt --package fret-ui-editor
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
```

## Residual Gaps

- Palette entries are app-provided but not user-editable inside the popup.
- Drag/drop-to-palette-slot mutation, color history, and eyedropper behavior stay in separate
  follow-ons.
