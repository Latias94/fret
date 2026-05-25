# IMUI Style Theme Editor Proof v1 - Handoff

Status: Closed
Last updated: 2026-05-25

Current slice: closed on 2026-05-25.

Boundary: keep editor style/theme tooling in `fret-ui-editor`. Do not move Dear ImGui global style
mutation APIs (`GetStyle`, `PushStyleVar`, `ImGuiStyle`) into `crates/fret-ui`, `fret-ui-kit::imui`,
or `fret-imui`.

Closeout:

1. This lane is implementation-complete and closed.
2. Use `EditorThemePresetPicker` for demo/workbench preset selection.
3. Start a new editor-owned follow-on for additional theme tooling.
