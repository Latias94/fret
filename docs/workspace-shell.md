# Workspace Shell (Editor Chrome) Direction

This repository targets editor-grade UI (Unity/Unreal/Godot workflows). In practice, the biggest
perceived gap versus Zed/GPUI is not the low-level UI substrate (`fret-ui`), but the lack of a
cohesive **workspace shell** layer:

- tab strip + pane headers,
- menubar / toolbar affordances,
- status bar,
- consistent theme tokens for “app chrome”,
- a predictable composition model around docking.

## Layering (anti-rewrite rule)

To avoid future rewrites, keep the following boundary strict:

- `crates/fret-ui`: **mechanism-only runtime substrate** (contracts; ADR 0066).
- `ecosystem/*`: **policy + editor shell** (fast iteration; may be moved out later).

The workspace shell is inherently policy-heavy (default sizes, interaction patterns, chrome
decisions), so it belongs in `ecosystem/`.

## Current Implementation

- `ecosystem/fret-workspace`: minimal workspace-shell building blocks:
  - `WorkspaceFrame`: top / center / bottom layout composition.
  - `WorkspaceTopBar`: a simple top bar layout (left / center / right slots).
  - `WorkspaceStatusBar`: a simple status bar layout (left / right slots).

This is intentionally small: it creates a stable integration seam without expanding the `fret-ui`
public contract surface.

## Recommended Next Steps (Windows-first)

1. Introduce an editor-grade `TabStrip` (not shadcn `Tabs`) with:
   - close / dirty indicators,
   - overflow scrolling,
   - keyboard navigation,
   - drag reorder (later),
   - pinned tabs (later).
2. Keep in-window menubar for cross-platform parity, but lock the future OS-menubar contract:
   - a data-only menu model already exists: `crates/fret-runtime/src/menu.rs`.
   - add an effect plumbing seam later (e.g. `Effect::SetMenuBar`) so native integration does not
     require rewriting UI-level menu authoring.
3. Wrap `fret-docking` inside `WorkspaceFrame` and unify chrome tokens (height/spacing/colors) so
   docking does not look like a separate subsystem.

