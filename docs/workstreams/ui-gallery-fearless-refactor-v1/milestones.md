# Milestones (UI Gallery fearless refactor v1)

## M1 — “Preview ≡ code” infrastructure

Exit criteria:

- Snippet modules exist under `apps/fret-ui-gallery/src/ui/snippets/`.
- `DocSection` can display code sourced from snippet files, including region slicing.
- Code blocks are scrollable and consistent in layout.

## M2 — Migrate high-traffic pages

Start with the pages most used for parity work and diagnostics.

Exit criteria:

- ButtonGroup page is snippet-backed.
- Select page is snippet-backed.
- At least 5 additional pages are migrated end-to-end.

## M3 — Parity discipline + regression gates

Exit criteria:

- A promoted diag suite runs a smoke pass across snippet-backed pages (existence selectors + one screenshot each).
- Each migrated page has stable `test_id` anchors for automation.
- For each mismatch class, a “where to fix it” note is captured (mechanism vs policy vs recipe).

## M4 — Authoring ergonomics

Exit criteria:

- A small helper (macro or function) makes it hard to wire preview/code inconsistently.
- Boilerplate in pages is reduced to a consistent pattern (render + include).

