# UI Gallery Fearless Refactor (Milestones)

This file defines milestones and acceptance criteria for the UI Gallery refactor described in
`plan.md`.

## Milestone 0 — Foundations

**Deliverables**

- `apps/fret-ui-gallery/src/ui/snippets/` folder created with a clear convention.
- A helper that allows:
  - Preview: `include!(...)` a snippet module and call `render(...)`.
  - Code tab: show `include_str!(...)` content.
- A documented pattern for “region slicing” (optional) to display only a portion of the snippet.
- Tracker table (`todo.md`) records upstream doc paths (Base + Radix) for each component.
- Snippets use user-facing imports (default: `use fret_ui_shadcn::prelude::*;`).

**Acceptance criteria**

- At least one page uses file-backed code for both preview and code.
- `cargo check -p fret-ui-gallery` succeeds.

## Milestone 1 — High-drift pages migrated

**Scope**

- Button Group page(s).
- Select page(s).

**Acceptance criteria**

- No `DocSection::code("rust", r#"...")` literals for the migrated sections.
- Example code shown in UI Gallery exactly matches preview behavior (by construction).
- Tracker rows for migrated components are updated with:
  - `Base MDX` and `Radix MDX` paths,
  - a short note about which doc variant is treated as primary for the snippet.

## Milestone 2 — Shadcn component breadth migration (batch-based)

**Scope**

- Migrate remaining shadcn component pages in batches (e.g. overlays first, then forms, then
  data-display).
- For each batch, update the tracker in `todo.md`.

**Acceptance criteria**

- ≥ 80% of shadcn component pages use snippet-backed code tabs.
- Any remaining string-literal code tabs have explicit “legacy” labels and are tracked.

## Milestone 3 — Enforcement + regression guards

**Deliverables**

- A lightweight enforcement mechanism for migrated pages:
  - Prefer a compile-time pattern (snippets compiled by default), and
  - Optionally a test that rejects new raw code literals in migrated modules.
  - Optional: a script/test that lists any remaining legacy code literals and fails CI once the
    workstream reaches a chosen threshold.

**Acceptance criteria**

- New drift is prevented by tooling, not review memory.

## Milestone 4 — Optional quality upgrades (post-migration)

These are optional and should only be tackled once drift is eliminated.

- Consistent page taxonomy and navigation IDs aligned with shadcn docs.
- “Copy/paste ready” mode: code snippets include required imports and model initialization.
- Better diagnostics integration:
  - stable `test_id` surfaces for key interactive examples,
  - optional `fretboard diag` scripts for high-risk overlay families.
