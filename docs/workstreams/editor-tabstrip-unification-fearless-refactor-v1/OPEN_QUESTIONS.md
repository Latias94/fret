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

## Q3: Do we need pinned tabs / multi-row?

Zed supports pinned/unpinned and can render separate rows.

Recommendation:
- Track as a separate milestone item (M0 parity map first). If we add it, keep it in `workspace`
  policy layer; headless only needs to understand surface classification + overflow membership.

## Q4: Where does “scroll active into view” live?

Recommendation:
- Mechanism: a headless helper that computes “required scroll delta to make rect visible”.
- Policy: when to call it (on active change, on dropdown selection, after drop).

