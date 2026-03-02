# Shadcn Component Surface Audit v1 (Milestones)

Last updated: 2026-03-02.

## M0 — Inventory + priority ordering

Deliverables:

- A tracker table covering every component in upstream Radix docs.
- A prioritized “audit order” that starts from the most contract-sensitive surfaces:
  - overlays (dialog/popover/dropdown/context menu)
  - composite widgets (select/combobox/tabs/menubar/navigation menu)
  - scroll/virtualization sensitive tables

Exit criteria:

- Every component is listed with `Not audited` / `In progress` / `Done (known gaps)` / `Done`.

## M1 — Part surface parity (top priority set)

Components (first batch):

- `select`, `combobox`
- `dropdown-menu`, `context-menu`, `menubar`, `navigation-menu`
- `dialog`, `popover`, `tooltip`

Exit criteria:

- Each component has:
  - one “surface completeness” note (what parts are missing / drifted)
  - one automation gate (at minimum: `test_id` stability)
  - one behavior gate (keyboard/focus or overlay dismiss) when it’s contract-sensitive

## M2 — Policy extraction (reuse-first)

Deliverables:

- Any reusable behavior helpers land in `ecosystem/fret-ui-kit` (not in `fret-ui`).
- Recipe crates keep shadcn defaults and composition only.

Exit criteria:

- Added helpers are covered by a kit-level unit test.
- Shadcn layer consumes helpers without re-implementing the same logic.

