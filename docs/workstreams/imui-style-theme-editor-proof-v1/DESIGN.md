# IMUI Style Theme Editor Proof v1 - Design

Status: Closed
Last updated: 2026-05-25

## Boundary

Dear ImGui has `ShowStyleEditor`, `GetStyle`, and scoped style mutation APIs. Fret should not copy
that shape into `crates/fret-ui`: the runtime theme contract is a mechanism layer, while editor
style defaults are component/ecosystem policy.

This lane adds the narrow Fret equivalent needed by the IMUI/editor workbench path:

- stable editor theme preset metadata,
- reversible preset switching,
- a declarative `EditorThemePresetPicker` control in `fret-ui-editor`,
- a thin `fret-ui-editor::imui::editor_theme_preset_picker` adapter,
- tests proving ListBox semantics, click-to-switch model updates, and theme replay.

## Non-Goals

- No `ImGuiStyle` clone.
- No `GetStyle`, `PushStyleVar`, or global mutable style stack in `fret-ui`.
- No theme editor implementation in `fret-ui-kit::imui`.
- No design-system palette editor or arbitrary token table editor.
- No shadcn/material policy changes.
- No `fret-imui` dependency growth.

## Implementation Shape

- `theme.rs` owns preset identity, labels, parsing, install/replay helpers, and reversible base
  numeric scrub defaults.
- `controls/editor_theme_preset_picker.rs` owns the editor control and keeps semantics as
  `ListBox` / `ListBoxOption`.
- `primitives/readout.rs` owns text-role props for the picker so editor controls do not accumulate
  direct `TextProps` literals.
- `src/imui.rs` remains a one-hop adapter over the declarative control.
- `imui_surface_policy.rs` keeps the adapter layer thin.

## Why This Is Dear ImGui-Aligned Without Copying Its Runtime Shape

The product outcome is the same first-step affordance: an editor user can inspect and switch
dense/editor style presets in the immediate authoring path. The architecture is intentionally
different: Fret patches editor-owned theme tokens through the existing theme system and remembers
the installed preset for host-theme replay, instead of exposing a mutable global style struct as a
runtime contract.
