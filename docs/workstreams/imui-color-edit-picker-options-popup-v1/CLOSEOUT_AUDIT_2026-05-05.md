# ImUi Color Edit Picker Options Popup v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `ColorEditPopupOptions::picker_options` as an explicit per-control policy switch.
- Added popup-local runtime option state for picker shape and AlphaBar visibility.
- Added a compact popup options surface with HueBar/HueWheel radio-style controls and an AlphaBar
  checkbox-style toggle.
- Kept hidden picker policy respected: a hidden app-owned picker default is not re-enabled by
  runtime option state.
- Kept ownership in `fret-ui-editor`; `fret-imui` remains a thin adapter and no global
  `SetColorEditOptions()` state path was introduced.

## Evidence

- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/options.rs`
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

- This lane uses an explicit popup-local options section instead of a right-click context menu.
- Color history, eyedropper behavior, palette customization, and full thumbnail-preview polish stay
  in separate follow-ons.
