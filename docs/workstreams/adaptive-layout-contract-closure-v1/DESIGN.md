# Adaptive Layout Contract Closure v1

Status: Active
Last updated: 2026-04-10

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `TARGET_INTERFACE_STATE.md`
- `M1_CONTRACT_FREEZE_2026-04-10.md`
- `BASELINE_AUDIT_2026-04-10.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
- `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- `docs/workstreams/container-queries-v1/container-queries-v1.md`
- `docs/workstreams/environment-queries-v1/environment-queries-v1.md`
- `docs/workstreams/genui-json-render-v1/genui-json-render-v1.md`
- `docs/known-issues.md`
- `docs/crate-usage-guide.md`

This lane exists because Fret already landed the lower-level adaptive mechanisms, but it still
does not have one active execution surface that answers the framework-level question:

> what should "adaptive UI" mean in Fret end-to-end, and which behaviors belong to container
> queries, environment queries, caller-owned shell sizing, or explicit strategy components?

The older `container-queries-v1` and `environment-queries-v1` lanes are still valuable contract
references, but they do not currently own:

- the public authoring taxonomy,
- the first-party UI Gallery teaching surface,
- the breakpoint vocabulary story on `fret::env`,
- or the next fearless-refactor slices for recipe/page drift.

This lane closes that gap without collapsing mechanism and policy back together.

## Assumptions-first baseline

### 1) Lane ownership

- Area: workstream ownership
- Assumption: this should be a new active execution lane rather than a reopening of the older
  implementation trackers.
- Evidence:
  - `docs/todo-tracker.md`
  - `docs/workstreams/container-queries-v1/container-queries-v1-milestones.md`
  - `docs/workstreams/environment-queries-v1/environment-queries-v1-milestones.md`
- Confidence: Likely
- Consequence if wrong: we would blur already-landed mechanism work with a broader authoring and
  gallery cleanup lane.

### 2) Query-axis split stays hard

- Area: contract boundary
- Assumption: container queries and environment queries must stay separate, even if they share
  familiar breakpoint token names.
- Evidence:
  - `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
  - `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Confidence: Confident
- Consequence if wrong: Fret would regress into an ambiguous "responsive helper" surface that is
  hard to audit and easy to misuse inside docking/panel-heavy apps.

### 3) The main gap is authoring closure, not a new runtime engine

- Area: owning layer
- Assumption: most of the missing work is in taxonomy, teaching surface, targeted recipe cleanup,
  and proof gates, not a new runtime-wide responsive engine.
- Evidence:
  - `docs/workstreams/container-queries-v1/container-queries-v1-milestones.md`
  - `docs/workstreams/environment-queries-v1/environment-queries-v1-milestones.md`
  - `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
  - `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
- Confidence: Confident
- Consequence if wrong: this lane will need to split a narrower mechanism follow-on instead of
  silently widening `crates/fret-ui`.

### 4) `fret::env` should remain the explicit app-facing import lane

- Area: public authoring surface
- Assumption: adaptive helpers should stay explicit on `fret::env::{...}` rather than drift into
  the default app prelude.
- Evidence:
  - `docs/crate-usage-guide.md`
  - `ecosystem/fret/src/lib.rs`
- Confidence: Confident
- Consequence if wrong: app code will lose the current "explicit opt-in" posture for adaptive
  behavior and make query-axis ownership harder to see during review.

### 5) First proof should stay on user-visible surfaces

- Area: proof surface
- Assumption: UI Gallery narrow-window behavior and an existing panel-resize demo are the right
  first proof surfaces before any larger refactor.
- Evidence:
  - `apps/fret-ui-gallery/tests/popup_menu_narrow_surface.rs`
  - `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json`
  - `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/demo.rs`
  - `apps/fret-examples/src/container_queries_docking_demo.rs`
- Confidence: Confident
- Consequence if wrong: this lane would start from abstract helper design instead of the surfaces
  where users first see adaptive drift.

### 6) Remaining drift is mixed recipe/page debt, not one single bug

- Area: current debt shape
- Assumption: the next adaptive pass must audit mixed debt across recipes, docs copy, gallery page
  shells, and a few raw viewport reads or duplicated breakpoint seams.
- Evidence:
  - `docs/known-issues.md`
  - `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
  - `ecosystem/fret-ui-shadcn/src/carousel.rs`
  - `ecosystem/fret-ui-shadcn/src/dialog.rs`
  - `ecosystem/fret-ui-shadcn/src/drawer.rs`
- Confidence: Likely
- Consequence if wrong: we could overfit the lane to one component and miss the real framework
  authoring gaps.

## In scope

- Freeze an adaptive taxonomy that keeps these axes explicit:
  - container width / height,
  - viewport or device capabilities,
  - caller-owned shell sizing,
  - strategy-layer adaptive components.
- Audit the current public authoring path:
  - `fret::env`,
  - `fret-ui-kit` helper surface,
  - `fret-ui-shadcn` recipe APIs,
  - first-party UI Gallery snippets and page notes,
  - GenUI adaptive strategy components.
- Pick the first fearless-refactor slices where current drift is reviewable and bounded.
- Leave one gallery narrow-surface proof and one panel-resize proof that future adaptive work must
  keep green.

## Out of scope

- Adding a CSS media-query parser or a general runtime "responsive engine".
- Moving policy defaults into `crates/fret-ui`.
- Rewriting every existing gallery page in one pass.
- Widening generic `children(...)` APIs just because a surface is adaptive; only widen APIs where
  copyable source evidence shows the current authoring path is insufficient.

## Owning layers

- `crates/fret-ui`
  - only for narrow mechanism follow-ons if the current query contracts prove insufficient
- `ecosystem/fret-ui-kit`
  - typed adaptive helpers, shared breakpoint vocabulary, and strategy-layer infrastructure
- `ecosystem/fret-ui-shadcn`
  - recipe defaults, responsive policy, and source-aligned adaptive authoring surfaces
- `ecosystem/fret`
  - explicit `fret::env` app-facing import lane
- `apps/fret-ui-gallery`
  - first-party teaching surface and narrow-window proof
- `apps/fret-examples` / `ecosystem/fret-genui-shadcn`
  - editor-grade panel proof and adaptive strategy exemplars

## Target shipped state

When this lane is done, the following must be true:

1. Fret has one explicit adaptive vocabulary that separates container queries from
   environment/device queries while keeping breakpoint naming predictable.
2. The first-party UI Gallery teaches which widths are caller-owned, which behaviors are
   container-driven, and which behaviors are environment-driven.
3. Remaining raw viewport magic numbers are either migrated behind helpers or explicitly audited as
   intentional device-level policy.
4. Narrow-window gallery demos do not rely on fixed-width shells that overflow the docs/detail
   layout by default.
5. At least one strategy-level adaptive exemplar stays tied to gates across both narrow-window and
   panel-resize proof surfaces.
6. Any mechanism change that survives this lane remains boundary-safe and updates ADR/alignment
   docs when it touches a hard contract.
