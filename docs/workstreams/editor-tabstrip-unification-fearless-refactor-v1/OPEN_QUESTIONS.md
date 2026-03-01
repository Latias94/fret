# Editor TabStrip Unification Fearless Refactor v1 (Open Questions)

## Q1: Overflow dropdown item policy

Options:
1) Overflowed-only (Dockview-style).
2) Overflowed + active tab (current Fret docking behavior).
3) All tabs (fallback-only, for debugging).

Recommendation:
- Keep policy in adapters; default to “overflowed + active” because it prevents a common editor
  failure mode where the active tab becomes unreachable in the dropdown.

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

## Q6: Where should shared tabstrip controller code live?

Options:
1) `fret-ui-headless` (mechanism) — maximally reusable, but risks pulling policy into the contract layer.
2) `fret-ui-kit` (policy toolbox) — shared implementation without expanding `fret-ui` contracts.
3) Keep duplicated per adapter — simplest short-term, but drifts quickly.

Recommendation:
- Put the **TabStripController** in `fret-ui-kit` (option 2).
- Keep pure geometry/helpers in `fret-ui-headless` (e.g. surface classification, canonical end-drop).

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
