# IMUI Style Theme Editor Proof v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Boundary Locked

Exit criteria:

- The lane explicitly rejects runtime-level `GetStyle` / `PushStyleVar` cloning.
- The lane uses existing editor theme helpers and `ColorEdit` rather than adding parallel IMUI
  component policy.

## M1 - Preset Picker Shipped

Exit criteria:

- [x] `EDITOR_THEME_PRESETS` and preset labels exist.
- [x] Dense-to-default switching resets numeric scrub tokens.
- [x] `EditorThemePresetPicker` exists as a declarative editor control.
- [x] `fret-ui-editor::imui::editor_theme_preset_picker` stays a one-hop adapter.

## M2 - Proof Closed

Exit criteria:

- [x] Focused control and theme tests pass through `fret-ui-editor` nextest.
- [x] Adapter smoke and thin-surface policy tests pass.
- [x] Source-policy gate rejects global style-stack drift.
- [x] Catalog, JSON, format, and whitespace gates pass or known warnings are recorded.

## M3 - Workbench Integration Closed

Exit criteria:

- [x] The canonical editor-notes inspector exposes the picker as
      `editor-notes-demo.inspector.theme-preset`.
- [x] The device-shell variant reuses the same inspector content and preset model path.
- [x] `fret-examples` preset parsing delegates to `EditorThemePreset::from_key`.
- [x] Workbench source-policy tests and `cargo check -p fret-demo --bin imui_editor_workbench_demo`
      pass with the picker mounted.
