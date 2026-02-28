# Workspace TabStrip (editor-grade) — Open Questions

## 1) What is the “source of truth” for UX parity?

Options:
- Zed: editor-grade pane/tab bar interactions (pinned tabs, drag, split-on-drop, context menus).
- Dockview: docking/tab drag invariants (drop targets, header space semantics, test coverage style).

Recommendation:
- Use Zed as the primary UX reference for *workspace/editor* tabs.
- Use Dockview as the reference for *docking-style* drop target semantics and test strategy.

Note:
- Prefer invariants-based gates (diagnostics snapshots) over screenshot baselines. Use screenshots as
  a temporary bridge when the invariant surface is not yet stable or expressible.

## 2) Where should reusable pieces live (without making tiny crates)?

Candidates:
- `ecosystem/fret-workspace`: editor/workspace policy (default home for tab strip).
- `ecosystem/fret-ui-kit`: reusable policy primitives (focus/roving/menus/dnd helpers).
- `ecosystem/fret-ui-headless`: pure logic that should be UI-framework-agnostic.

Recommendation:
- Keep `WorkspaceTabs` + `WorkspaceTabStrip` in `ecosystem/fret-workspace`.
- Extract only clearly reusable helpers (e.g. “tab strip geometry kernel”, “edge autoscroll policy”)
  into modules under `ecosystem/fret-ui-kit` or `ecosystem/fret-ui-headless` (module-level reuse,
  not a new crate).

## 3) Should Workspace tabs support OS-window tear-off?

Considerations:
- Docking already owns cross-window/tear-off arbitration.
- Workspace tabs are “documents”; tear-off may become a docking concern (panel-level).

Recommendation:
- Do not add OS-window tear-off to workspace tabs in M1/M2.
- If needed later, integrate by hosting workspace panes inside the docking system rather than
  duplicating cross-window drag semantics.

## 4) What is the contract for drag/drop ownership?

Open questions:
- Should the tab strip output “intents” (headless) and let the shell decide commands?
- Or should the strip dispatch commands directly (current approach)?

Recommendation:
- Move toward an intent surface (activate/close/reorder/pin/unpin/start-drag).
- Keep `CommandId` dispatch as a thin adapter for demos and early apps.

## 5) Pinned tabs: boundary vs separate row

Open questions:
- Do we want both modes?
- If both: is it a setting or a compile-time choice?

Recommendation:
- Support both modes, keep default as “single row + pinned boundary” (simpler).
- Add a “separate pinned row” mode once geometry/overflow is stable.
