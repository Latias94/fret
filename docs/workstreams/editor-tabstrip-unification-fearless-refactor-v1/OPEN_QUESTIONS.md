# Editor TabStrip Unification Fearless Refactor v1 (Open Questions)

## Q1: Overflow dropdown item policy

Options:
1) Overflowed-only (Dockview-style).
2) Overflowed + active tab (current Fret docking behavior).
3) All tabs (fallback-only, for debugging).

Recommendation:
- Keep policy in adapters; default to “overflowed + active” because it prevents a common editor
  failure mode where the active tab becomes unreachable in the dropdown.
  - Docking default: includes active (`ecosystem/fret-docking/src/dock/tab_overflow.rs`).
  - Workspace default: overflowed-only (dockview-like), but can opt into include-active if needed
    (`WorkspaceTabStrip::overflow_menu_active_policy(...)`).

## Q2: Should overflow dropdown support close buttons?

Dockview has explicit tests for “overflow dropdown with close buttons”.

Recommendation:
- Yes for editor-grade UX, but implement as policy in docking/workspace layers, not headless.
- Docking + workspace are aligned (overflow dropdown rows expose a close affordance; close does not activate).

## Q3: Do we need pinned tabs / multi-row?

Zed supports pinned/unpinned and can render separate rows.

Recommendation:
- Track as a separate milestone item (M0 parity map first). If we add it, keep it in `workspace`
  policy layer; headless only needs to understand surface classification + overflow membership.

## Q4: Where does “scroll active into view” live?

Recommendation:
- Mechanism: a headless helper that computes “required scroll delta to make rect visible”.
- Policy: when to call it (on active change, on dropdown selection, after drop).
- Diagnostics/gates: treat “active tab is visible” as a first-class invariant for editor-grade UX
  and lock it with a `fretboard-dev diag` gate (non-pixel, no semantics required).
  - Current docking gate: `dock_tab_strip_active_visible_is visible=true` after selecting from
    the overflow dropdown.
  - Current workspace gate: `workspace_tab_strip_active_visible_is visible=true pane_id="pane-a"`
    after selecting from the overflow dropdown.

## Q5: Input arbitration priority (close vs activate vs overflow vs other affordances)

Observed bug class: hit targets can overlap (e.g. docking's float-zone affordance overlaps the tab
bar corner), so the ordering must be explicit and tested.

Recommendation:
- Treat this as adapter policy, but document and test the priority ordering.
- Proposed ordering for editor-grade tab UX:
  1) overflow dropdown/menu surface (rows, close vs content)
  2) overflow control button (toggle)
  3) tab close affordance
  4) tab activation (content)
  5) non-tabstrip affordances in the same corner (e.g. float-zone)

Additional note:
- Dockview prevents default on the close affordance `pointerdown` to suppress tab drag/activation
  when the intent is "close without activation". Fret adapters should implement an equivalent
  arbitration rule and lock it with a diag gate.
  - Workspace gate: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-does-not-start-drag.json`
  - Docking gate: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-close-button-does-not-activate.json`

## Q10: Should close affordances tolerate small pointer jitter (click slop)?

Observed bug class:
- Pointer-down begins on the close hit rect, but a small move causes pointer-up to miss the hit rect.
- Result: close is canceled (and may fall back to activation/drag depending on adapter logic).

Recommendation:
- Yes: treat close as an explicit intent and accept a small “click slop” window.
- Put the pure math in `fret-ui-headless` (e.g. `pointer_move_within_slop(start, end, slop)`), and keep the constant
  and arbitration wiring in adapters (`fret-workspace` / `fret-docking`) so we can tune it without expanding
  `fret-ui` contract surface.
- Lock it with a diag gate that intentionally includes pointer movement (e.g. 8px) while asserting:
  - close still happens,
  - activation does not happen,
  - drag does not start.

## Q6: Where should shared tabstrip controller code live?

Options:
1) `fret-ui-headless` (mechanism) — maximally reusable, but risks pulling policy into the contract layer.
2) `fret-ui-kit` (policy toolbox) — shared implementation without expanding `fret-ui` contracts.
3) Keep duplicated per adapter — simplest short-term, but drifts quickly.

Recommendation:
- Keep the **TabStripController** in `fret-ui-headless` (small shared arbitration rules).
- Re-export from `fret-ui-kit` for adapter convenience.
  - Code: `ecosystem/fret-ui-kit/src/headless/tab_strip_controller.rs`

## Q7: How should dropdown menu rows support trailing “action slots” (close, pin, etc.)?

Observed constraint:
- `DropdownMenuItem` is a row-level `Pressable`; nested pressables in `trailing(...)` do not
  currently behave as independent hit targets in all scenarios.

Current implementation:
- `fret-ui-shadcn` supports a trailing action command + hit region on `DropdownMenuItem`.
  - Evidence: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`

Open questions:
- Should trailing actions fire on pointer-down (current) or pointer-up (click/activate)?
- Do we need keyboard-accessible trailing actions (separate focus target / APG-aligned semantics)?
- Should this stay a `fret-ui-shadcn`-only capability, or be generalized in a headless menu primitive?

## Q8: What does “visible” mean for the active tab?

Options:
1) Fully visible (tab bounds entirely inside the strip viewport).
2) Partially visible (tab bounds intersects the strip viewport).

Recommendation:
- Use (2) for diagnostics gates to avoid false failures when a tab is wider than the viewport.
- Keep “fully visible” as a UI/behavior goal where feasible, but do not over-constrain early gates.

## Q9: Which upstream reference should “win” when behaviors conflict?

Options:
1) `repo-ref/zed` (editor-first, MRU/pinned, strong UX expectations).
2) `repo-ref/dockview` (docking-first, explicit tests for overflow/drop semantics).
3) `repo-ref/gpui-component` (element/controller patterns; declarative inspiration).

Recommendation:
- Prefer Dockview as the source of truth for docking chrome invariants (overflow membership,
  dropdown affordances, drop target semantics).
- Prefer Zed as the source of truth for editor interaction expectations (MRU fallback, focus
  behavior, “active stays reachable” ergonomics).
- Use GPUI references opportunistically for implementation patterns (stable keys, controller split),
  but do not treat it as the final behavior oracle.
