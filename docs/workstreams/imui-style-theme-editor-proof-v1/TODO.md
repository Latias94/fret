# IMUI Style Theme Editor Proof v1 - TODO

Status: Closed
Last updated: 2026-05-25

## STE-010 - Boundary Decision

- [x] Keep Dear ImGui style editor parity in `fret-ui-editor`, not `crates/fret-ui`.
- [x] Reuse the existing `ColorEdit` policy surface instead of adding a second color editor under
      `fret-ui-kit::imui`.
- [x] Treat this as preset/theme tooling, not an arbitrary token table editor.

## STE-020 - Preset Metadata And Reversibility

- [x] Add stable preset ordering and labels.
- [x] Add `from_key` parsing for tool/demo routing.
- [x] Expose installed preset lookup.
- [x] Add default numeric scrub tokens so switching back from dense clears dense-only overrides.

## STE-030 - Picker Control And IMUI Adapter

- [x] Add `EditorThemePresetPicker` and options in `fret-ui-editor`.
- [x] Stamp ListBox/ListBoxOption semantics and stable diagnostics ids.
- [x] Update preset model on activation and install/replay preset on render through editor theme
      helpers.
- [x] Add the thin `fret-ui-editor::imui::editor_theme_preset_picker` adapter.

## STE-040 - Proof

- [x] Add tests for preset metadata and dense-to-default reversibility.
- [x] Add control tests for semantics and click-to-switch replay.
- [x] Update IMUI adapter smoke and surface policy tests.
- [x] Add source-policy markers preventing global style-stack drift.
- [x] Run focused Rust/source/catalog/format gates and record evidence.

## STE-050 - Canonical Workbench Integration

- [x] Mount `EditorThemePresetPicker` in the canonical editor-notes inspector with a stable test id.
- [x] Reuse the installed preset as the initial model in both the canonical workbench and the
      device-shell variant.
- [x] Route examples preset parsing through `EditorThemePresetV1::from_key`.
- [x] Add source-policy tests and gates proving the picker stays in editor/examples policy, not
      runtime mechanism or `fret-ui-kit::imui`.
