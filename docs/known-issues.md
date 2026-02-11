# Known Issues / Diagnostics / Platform Limitations

This document collects **current known limitations** and **common diagnostics** so they don’t get lost inside
ADRs or the main docs entrypoints.

If you are new to the project, still start from `docs/README.md`.

## UI Kit Notes

- `docs/archive/backlog/ui-kit-gap.md`
- Review-driven TODOs (not necessarily user-facing limitations yet): `docs/todo-tracker.md`

### Container-query-driven responsive variants

Current behavior:

- The initial set of shadcn-aligned "responsive" variants that are container-query-driven upstream
  now use Fret's **container query** mechanism (ADR 0231).
- Remaining viewport-width breakpoints in recipe code should be treated as **device-level**
  behavior (e.g. "mobile vs desktop"), not as a substitute for container queries.

Impact:

- In docking/panel-heavy layouts, responsive behavior can drift because panel width is not the same
  as window width.

Plan:

- Keep new responsive additions on the container query path defined in ADR 0231:
  - ADR: `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
  - Workstream: `docs/workstreams/container-queries-v1.md`

## Common Diagnostics

### `WARN fret_ui::elements: unkeyed element list order changed`

Meaning:

- A dynamic list/tree was rendered **without explicit keys**, and the child order changed between frames.
- Element-local state (caret/selection/scroll/etc.) may “stick to positions” rather than to logical items.

What to do:

- Treat this as a correctness warning for anything dynamic:
  - **Any list/tree/table whose order can change must be keyed.**
  - Use `ElementContext::keyed(...)` / `ElementContext::for_each_keyed(...)` for dynamic collections.
  - For virtualized lists, prefer `ElementContext::virtual_list_keyed(...)` so each visible row is
    automatically scoped under a stable key.
  - Avoid `ElementContext::for_each_unkeyed(...)` unless the collection is static and never reorders.

Practical key sources (pick a stable one per domain):

- Engine/editor entities: stable `EntityId`/`Guid`.
- Assets: stable GUID (ADR 0026) or a stable asset handle (not a path).
- UI models: stable model IDs / node IDs (not indices).
- Files: path is acceptable only if you don’t need stable identity across renames/moves.

Reference:

- `docs/adr/0028-declarative-elements-and-element-state.md`

### Windows: `thread 'main' has overflowed its stack` when launching demos

Meaning:

- Windows executables may default to a small **main-thread stack reserve** (commonly 1 MiB).
- Deep recursive layout/hit-testing paths (e.g. taffy traversal, large trees) can overflow that stack.

What to do:

- Ensure demo executables are rebuilt after pulling:
  - We bump the Windows stack reserve for native `apps/` binaries via per-crate `build.rs`.
- Optional runtime fallback (desktop-only):
  - Set `FRET_STACKSAFE=1` to configure `stacksafe` early in `fret-launch` (defaults: `min=2MiB`, `alloc=8MiB`).

Related env vars:

- `FRET_WINDOWS_STACK_RESERVE_BYTES`: override the linker stack reserve for `apps/` binaries.
- `FRET_STACKSAFE_MIN_BYTES` / `FRET_STACKSAFE_ALLOC_BYTES`: tune `stacksafe` sizes.

### Windows: `cargo build -p fret-demo --bins` fails with OOM / invalid metadata

Meaning:

- Building many statically-linked wgpu demo binaries in parallel can exhaust virtual memory on Windows.
- Symptoms include `LNK1102: out of memory` and Rustc metadata mmap failures (`os error 1455`).

What to do:

- Use fewer build jobs:
  - `python3 tools/windows/build-fret-demo-bins.py` (recommended).
- If needed, override via `CARGO_BUILD_JOBS` or increase the system page file.

## Rendering Artifacts (Notes)

- Pixelate backdrop clear-colored holes: [`docs/known-issues/effects_pixelate_holes.md`](known-issues/effects_pixelate_holes.md)

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
