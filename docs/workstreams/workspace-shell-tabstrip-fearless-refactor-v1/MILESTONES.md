# Workspace Shell TabStrip (Fearless Refactor v1) — Milestones

This workstream is staged so that every step leaves behind a gate (test/diag) and does not violate
layering (mechanism in `crates/*`, policy in `ecosystem/*`).

## M0 — Baseline + gates (done/verified)

**Outcome**

- Roving keyboard navigation is installed for workspace tab strips.
- Wheel over the tab strip scrolls horizontally when the strip overflows.
- Pointer-down on tabs does not steal focus from the content area.

**Gates**

- nextest: `-p fret-ui` roving flex tests
- nextest: `-p fret-workspace` tab strip focus stability test

## M1 — Overflow dropdown/list (dockview-inspired)

**Outcome**

- When the tab strip overflows, show an overflow control (e.g. “More” button).
- Overflow list renders overflowed tabs (fallback: list all tabs if overflow geometry is not yet available).
- Selecting a tab from the overflow list activates it and scrolls it into view.
- Optional: close buttons inside overflow list.

**Evidence anchors**

- dockview overflow pipeline: `repo-ref/dockview/.../tabs.ts` + `tabsContainer.ts`
- Zed behavior reference: `repo-ref/zed/.../pane.rs` (drop targets + scroll policies)
- Fret adapter + overflow UI: `ecosystem/fret-workspace/src/tab_strip.rs`
- Fret overflow computation helper: `ecosystem/fret-workspace/src/tab_strip_overflow.rs`

**Gates**

- Unit test: overflow membership computation is stable under viewport changes.
- Integration test: overflow menu opens and renders deterministic entries.
- Diag script (preferred, TODO): open overflow list, select an overflowed tab, assert active tab test id.

## M2 — Pinned boundary + (optional) separate pinned row (Zed-inspired)

**Outcome**

- Support pinned tabs:
  - a pinned boundary (`pinned_tab_count` or per-tab flag),
  - drop targets that allow pin/unpin moves.
- Optional (if worth it early): separate pinned row when both pinned + unpinned exist.

**Evidence anchors**

- Zed pinned rows + drop targets: `repo-ref/zed/crates/workspace/src/pane.rs`
- Fret pinned model + commands: `ecosystem/fret-workspace/src/tabs.rs` and `ecosystem/fret-workspace/src/commands.rs`

**Gates**

- Unit test: pin/unpin operations preserve active tab and do not corrupt indices.
- UI-level test/diag: dragging a tab to the pinned boundary results in pinned ordering.

## M3 — Preview tab semantics (Zed/VS Code style)

**Outcome**

- Support preview tab:
  - opening a “previewable” item reuses/replaces existing preview tab,
  - converting preview → normal preserves ordering and active tab.

**Evidence anchors**

- Zed preview tab comments/logic: `repo-ref/zed/crates/workspace/src/pane.rs` (`preview_tabs`)

**Gates**

- Unit test: opening sequence produces expected preview replacement behavior.

## M4 — Drag reorder + cross-pane move consolidation

**Outcome**

- One shared interaction kernel drives:
  - reorder insertion (before/after),
  - “drop into header space” insertion at end,
  - cross-pane move intents (workspace panes and docking panes).
- Docking tab bars adopt the same kernel (no duplicate reorder policy).

**Gates**

- Unit test: reorder intent is correct for a matrix of rects + pointer positions.
- Integration test: cross-pane drop updates active tab and preserves pinned boundary.

## M5 — Drag-to-split (edge targets) integration

**Outcome**

- Dragging a tab near pane edges offers split drop targets (if allowed by the shell).
- Dropping executes a split + move (Zed-like outcome).

**Constraints**

- Split authorization is policy-owned by workspace/docking; kernel emits intent only.

**Gates**

- Diag script: drag tab to edge, drop, assert a second pane exists and tab moved.
