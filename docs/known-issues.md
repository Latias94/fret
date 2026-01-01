# Known Issues / Diagnostics / Platform Limitations

This document collects **current known limitations** and **common diagnostics** so they don’t get lost inside
ADRs or the main docs entrypoints.

If you are new to the project, still start from `docs/README.md`.

## UI Kit Notes

- `docs/archive/backlog/ui-kit-gap.md`
- Review-driven TODOs (not necessarily user-facing limitations yet): `docs/todo-tracker.md`

## Common Diagnostics

### `WARN fret_ui::elements: unkeyed element list order changed`

Meaning:

- A dynamic list/tree was rendered **without explicit keys**, and the child order changed between frames.
- Element-local state (caret/selection/scroll/etc.) may “stick to positions” rather than to logical items.

What to do:

- Treat this as a correctness warning for anything dynamic:
  - **Any list/tree/table whose order can change must be keyed.**
  - Use `ElementCx::keyed(...)` / `ElementCx::for_each_keyed(...)` for dynamic collections.
  - For virtualized lists, prefer `ElementCx::virtual_list_keyed(...)` so each visible row is
    automatically scoped under a stable key.
  - Avoid `ElementCx::for_each_unkeyed(...)` unless the collection is static and never reorders.

Practical key sources (pick a stable one per domain):

- Engine/editor entities: stable `EntityId`/`Guid`.
- Assets: stable GUID (ADR 0026) or a stable asset handle (not a path).
- UI models: stable model IDs / node IDs (not indices).
- Files: path is acceptable only if you don’t need stable identity across renames/moves.

Reference:

- `docs/adr/0028-declarative-elements-and-element-state.md`

## Platform Limitations (Current)

### External OS file drag & drop on macOS (winit)

Current behavior:

- With winit today, macOS file DnD often provides only “enter” and “drop” style events and lacks a continuous
  drag-over callback with cursor position.
- That makes “per-widget drop target hover” inherently best-effort on macOS in the current backend.

Impact:

- You may see the app log enter/drop events, but UI hit-testing for “drag hover” targets won’t behave like
  Unity/ImGui on macOS until the backend improves.

Plan:

- Treat “native external DnD backend (macOS/Windows) with DragOver position” as a future platform task once
  core editor workflows (docking, viewports, text) are solid.
- In the meantime, treat external file drops as a **window-level drop** (best-effort hit-test on drop only),
  and keep “rich hover previews / accept/reject feedback” as an internal drag-session feature (ADR 0041).

Reference:

- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

Related capability:

- `PlatformCapabilities.dnd.external_position == "best_effort"` indicates hover cursor positions are not
  reliable for external OS drags; components should avoid committing to rich “drag hover” UX and instead
  treat drop targeting as best-effort (e.g. resolve target on drop only).
