# Workspace TabStrip (editor-grade) v1 — Gap Analysis

This document compares our current `WorkspaceTabStrip` implementation against upstream references
and turns “it feels incomplete” into a prioritized, gateable backlog.

The goal is not to port DOM/CSS implementations 1:1. The goal is to align **editor outcomes** and
lock them behind **tests + diag scripts** so refactors remain fearless.

## Reference mapping (what to copy from whom)

- **Zed (`repo-ref/zed`)** — primary source for editor semantics:
  - pinned boundary + optional separate pinned row,
  - preview tab lifecycle + “commit” semantics,
  - close policies that respect pinned/preview rules,
  - tab-bar buttons / nav history affordances,
  - “activate then keep typing” focus invariants.
- **dockview (`repo-ref/dockview`)** — primary source for overflow + drop-surface mechanics:
  - overflow membership + overflow list pipeline,
  - treating “header/void space” as a first-class drop surface,
  - invariants-first unit tests for tab containers.
- **gpui-component (`repo-ref/gpui-component`)** — useful for GPUI wiring patterns:
  - dock/tab panel composition patterns,
  - focus proxying via `Focusable` / `FocusHandle` delegation,
  - explicit “tab group” focus grouping.

Practical rule:

- If it affects **editor workflows** (pinned/preview/split/focus), align to **Zed outcomes** first.
- If it’s about **overflow/drop-surface UX and tests**, align to **dockview outcomes** first.
- If it’s about **how to wire** a dock/tab surface in a GPUI-ish architecture, consult
  **gpui-component** for patterns.

## Where Fret is already strong (today)

We already have several “editor-grade” outcomes implemented (and many are gated):

- Focus neutrality on pointer-down (chrome should not steal editor focus).
- Middle-click close (policy-owned in tab strip interaction).
- Context menu actions (close/pin/split variants; feature-flagged menu surface).
- Reorder + end-drop target (`insert_index == tab_count`) and header-space “drop at end”.
- Cross-pane tab drag intents and drag-to-split integration (workspace shell demos + scripts).
- Preview + pinned boundary semantics in `WorkspaceTabs` and surfaced in `WorkspaceTabStrip`.
- Keyboard roving baseline (APG-style roving for ArrowLeft/ArrowRight via `roving_nav_apg`).
- Shell-level focus transfer commands:
  - `workspace.pane.focus_tab_strip`
  - `workspace.pane.focus_content`
  - `workspace.pane.toggle_tab_strip_focus` (default `Ctrl+F6`)
- Exit fallback seam: `WorkspacePaneContentFocusTarget` lets `focus_content`/toggle exit the tab
  strip even when no explicit “return focus” target was recorded.

## Gaps vs Zed (editor semantics)

These gaps are about “what editor users expect” rather than raw feature count.

### P0 (blocks editor-feel)

- **Separate pinned row option**:
  - Zed supports a “pinned row” mode; we currently only have a single-row boundary.
  - Risk: once apps depend on single-row layout, multi-row will be a disruptive refactor.
  - Gate idea: a diag script + unit invariants for pinned row layout (no overlap + stable hit
    testing across the row boundary).

- **Tab bar button cluster outcomes** (nav history, new/split/zoom buttons):
  - Zed treats these as part of the tab bar surface and they influence hit-testing and overflow.
  - In Fret, these belong to the workspace shell policy layer (not `fret-ui`).
  - Gate idea: stable `test_id` anchors for left/right slots and “header space” drop surfaces that
    account for those slots.

### P1 (important, but can ship after P0)

- **Close policies with pinned/preview constraints**:
  - Zed has nuanced “close pinned?” options for batch close actions.
  - We have commands and UI affordances, but we should lock policy explicitly (what is allowed to
    close, and when) and gate it via unit tests.

- **Scroll-to-active rules** under mixed pinned/overflow:
  - Zed scrolls active tab into view with predictable rules (especially when active is pinned).
  - We should lock “when do we scroll” and “which viewport rect is authoritative” (measured vs
    cached) with stress gates.

### P2 (polish / later)

- Tooltips, tab title truncation rules, and width stability under dirty/preview transitions.
- Tear-out / popout / multi-window tab ownership transitions (future workspace-level workstream).

## Gaps vs dockview (overflow + drop-surface pipeline + tests)

Dockview’s key advantage is not styling — it’s the **pipeline** and the **tests**.

### P0

- **First-class “void/header space” surface**:
  - Dockview has a dedicated `VoidContainer` that participates in drag-over, drop, and overlay
    routing (`header_space` overlay kind).
  - We have an end-drop target, but we should treat “non-tab header space” as a named surface with
    explicit invariants (especially once we add tab-bar buttons).

### P1

- **Overflow observer discipline**:
  - Dockview uses an observer (`OverflowObserver`) + scroll listeners to drive overflow list state.
  - In Fret, overflow membership is derived from measured rects; we should ensure the measurement
    cache and invalidation rules are deterministic under resize/scroll/animation.
  - Gate idea: scripted “resize-scroll-resize” loop with assertions that overflow membership and
    activation remain stable.

## Gaps vs gpui-component (wiring & focus patterns)

gpui-component is not an editor tab-bar reference, but it has useful “wiring” patterns:

- **Focusable delegation**: `TabPanel` returns the active panel’s `FocusHandle`, which makes “focus
  the panel” deterministic without exposing inner widget IDs.
- **`tab_group()` semantics**: a clear focus-group boundary for keyboard navigation.

In Fret, we currently solve some of this via a `GlobalElementId` registry + command scope policy.
That is acceptable for v1, but for docking reuse we may eventually want a more explicit “focus proxy”
pattern to reduce the amount of cross-frame element-id bookkeeping.

## Recommended “fearless refactor” path (next)

1) **Document & gate the “pane content focus target” contract**
   - Make `WorkspacePaneContentFocusTarget` the recommended escape hatch for shells.
   - Add a small integration example in a workspace shell (and a diag script) once implemented.

2) **Continue kernel/adapter separation**
   - Keep policy in `ecosystem/fret-workspace`.
   - Extract only generic, headless math to `ecosystem/fret-dnd` (already the pattern).

3) **Treat header space as a named surface**
   - Explicitly model “tab strip row”, “header space”, and “overflow control” as separate hit-test
     regions so future button clusters don’t destabilize drop targeting.

4) **Add the separate pinned row option**
   - Prefer adding it behind a workspace policy toggle, not as a permanent fork in rendering code.

5) **Docking reuse pass**
   - Once the kernel seams are stable, apply the same kernel to `ecosystem/fret-docking` tab bars.

