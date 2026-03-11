# UI Gallery Fearless Refactor (Milestones)

This file defines milestones and acceptance criteria for the UI Gallery refactor described in
`plan.md`.

## Status

As of 2026-03-11:

- Milestones 0–3 are effectively complete in-tree (snippet-backed pages + enforcement tests + drift audit).
- Milestone 4 remains optional / follow-up (taxonomy polish, consistency, etc).
- The workstream is now in stabilization / polish; the remaining items are bounded and no longer
  require interface-level refactors.
- The tracked TODO checklist is functionally complete; remaining work is optional follow-up rather
  than migration debt.
- AI Elements demos are snippet-backed (see `ai-elements-tracker.md`).
- Material 3 pages are snippet-backed and routed through `src/ui/pages/material3/mod.rs`; see
  `material3-tracker.md` for the retirement record and any future polish work.
- Main form/trigger controls now have focused label-association closure (stable automation anchors + dedicated diag gates) for `Select`, `NativeSelect`, `Slider`, `RadioGroup`, `ToggleGroup`, `Combobox`, `DatePicker`, `Switch`, `Input`, `Textarea`, and `Toggle`.
- Authoring-surface cleanup is complete on the internal preview layer:
  - `src/ui/pages/**` is fully on `UiCx`,
  - `src/ui/content.rs` and `src/ui/nav.rs` are on `UiCx`,
  - `src/ui/previews/**` is fully on `UiCx` and guarded by dedicated `tests/ui_authoring_surface_*.rs` source gates,
  - `0 / 92` preview-surface files need migration off `ElementContext<'_, App>`.
- The follow-up deletion pass is underway:
  - dead `*_legacy` helpers in the first cleaned preview buckets are removed,
  - orphan `src/ui/previews/gallery/data/table*.rs` bridge files are removed,
  - the legacy Material 3 preview tree is removed.
- The next cleanup lane is also active:
  - feature-gated preview/page entry points are being aligned so default builds stop compiling
    unreachable dev/material3 wiring,
  - default workspace bootstrap no longer seeds dev-only tabs or a dev-only diagnostics start page
    when `gallery-dev` is disabled,
  - crate-local warning cleanup for `fret-ui-gallery` is currently complete:
    - `fret-ui-gallery` warnings under `cargo check -p fret-ui-gallery --lib`: 0,
    - `fret-ui-gallery` warnings under
      `cargo check -p fret-ui-gallery --lib --features gallery-full`: 0.

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
  - focused label-association gates for the main form/trigger controls,
  - optional `fretboard diag` scripts for high-risk overlay families.
- Internal teaching surfaces under `src/ui/previews/**` converge on `UiCx` before we delete the
  remaining legacy preview helpers.
- Reduce gallery-internal dead code (`doc_layout` helpers, orphan snippet assets, legacy Material 3
  preview layer) so warning counts track live authoring surfaces rather than historical scaffolding.
  - 2026-03-11 retirement result: `src/ui/previews/material3.rs` +
    `src/ui/previews/material3/**` were confirmed orphaned / not compiled and have been deleted.
- DocSection chrome/layout audit result: remaining wide-page overrides are intentional and tracked
  in `layout-audit.md`; no required width-normalization work remains on the main workstream path.
