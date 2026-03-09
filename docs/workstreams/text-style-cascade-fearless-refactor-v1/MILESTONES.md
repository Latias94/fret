# Text Style Cascade (Fearless Refactor v1) — Milestones

Status: In progress.

Primary contract: `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`

## Milestone 0 — Contract lock + audit

Goal:

- lock the v1 contract,
- capture the migration surface,
- avoid future redesign churn.

Deliverables:

- ADR 0314 accepted.
- Workstream docs created.
- Initial component enhancement / migration matrix recorded.

Exit criteria:

- The repo has one stable answer for “what inherited text style means in v1”.

## Milestone 1 — Portable refinement model

Goal:

- add a mergeable text refinement type without yet changing broad component behavior.

Deliverables:

- `TextStyleRefinement` (or equivalent) in the portable text contract.
- Merge/refine semantics documented and tested.

Exit criteria:

- There is a stable, partial text-style data model that subtree defaults can use.

## Milestone 2 — Runtime inherited text-style path

Goal:

- make inherited text style real in `crates/fret-ui` for passive text leaves.

Deliverables:

- inherited text-style propagation in runtime traversal,
- passive text leaves resolve `explicit > inherited > theme default`,
- measurement/cache integration tests.

Exit criteria:

- Passive text no longer needs component-local recursive patching to inherit typography.

## Milestone 3 — `fret-ui-kit` authoring surface

Goal:

- make the mechanism ergonomic enough that ecosystems stop hand-rolling it.

Deliverables:

- subtree text-style helper APIs,
- preset bridge from `ui-typography-presets-v1`,
- initial authoring guidance.

Exit criteria:

- Component authors have a preferred helper path instead of leaf-level manual assembly.

Status note:

- Landed in `ecosystem/fret-ui-kit/src/typography.rs` with subtree helpers, a preset-to-refinement bridge, and guidance to pair subtree defaults with `ui::raw_text(...)` when a leaf should stay unstyled.

## Milestone 4 — High-value component migration

Goal:

- migrate the most duplication-heavy families first.

Priority families:

- shadcn description family:
  - `AlertDescription`
  - `DialogDescription`
  - `SheetDescription`
  - `PopoverDescription`
  - `CardDescription`
  - `FieldDescription`
- AI direct-children pressure point:
  - `ConfirmationTitle`

Deliverables:

- migrated components (`AlertDescription`, `DialogDescription`, `SheetDescription`, `PopoverDescription`, `CardDescription`, `FieldDescription`, `ConfirmationTitle`),
- focused regression tests,
- removal of the first temporary local workaround.

Exit criteria:

- The description family no longer duplicates its typography inheritance story per component.

Status note:

- The shadcn description family and `ConfirmationTitle` now use the shared `fret-ui-kit` typography helpers; `TSC-kit-024` is now decided as a selective component-layer `children` API, with `AlertDescription`, `CardDescription`, and `DialogDescription` as the first adopters.
- Visual-text recipes that need late-bound resolved typography (currently `Shimmer`) are tracked separately in `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/`.

## Milestone 5 — Cleanup and adoption guardrails

Goal:

- reduce long-tail drift and lock the new path in.

Deliverables:

- cleanup of now-redundant component-local patch code,
- cleanup of duplicated description metric lookup logic,
- optional lint/grep guidance or review checklist to discourage new ad hoc patterns.

Exit criteria:

- The repo teaches one boring subtree-typography path and no longer depends on tactical patches for
  core description/body surfaces.
