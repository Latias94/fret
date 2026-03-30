# imui sortable recipe v1 - milestones

Status: closed closeout record

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-sortable-recipe-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-sortable-recipe-v1/TODO.md`

Closeout audit:

- `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Decision note:

- `docs/workstreams/imui-sortable-recipe-v1/SECOND_PROOF_SURFACE_DECISION_2026-03-30.md`

Predecessor closeout:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`

Closeout reading rule on 2026-03-30:

- treat this file as the historical progress record for the now-closed first immediate sortable
  recipe lane
- read `CLOSEOUT_AUDIT_2026-03-30.md` for the shipped outcome and surviving defer list
- read `SECOND_PROOF_SURFACE_DECISION_2026-03-30.md` for the proof-surface closure decision

## Phase A - Workstream setup and owner freeze

Status: Completed

Goal:

- open the follow-on lane explicitly,
- freeze the owner split before new code lands,
- and keep sortable policy out of `fret-ui-kit::imui`.

Deliverables:

- one new workstream directory with design/TODO/milestones,
- one explicit owner split for `imui` seam vs recipe layer vs optional `fret-dnd` helpers,
- one README/workstream-map update that points readers to this lane.

Exit gates:

- the repo can explain why this is a recipe lane rather than another `imui` helper lane,
- and the first proof/gate package is named before implementation starts.

## Phase B - Minimal sortable recipe contract

Status: Completed

Goal:

- freeze the smallest reusable sortable contract worth shipping.

Deliverables:

- one first-slice scope decision for vertical list/outliner reorder,
- one explicit responsibility split between app state and recipe state,
- one decision on whether any tiny insertion helper belongs in `fret-dnd`.

Exit gates:

- the first contract does not accidentally include multi-container or shell-specific semantics,
- the recipe can be explained in terms of the already-shipped `imui` drag/drop seam,
- and any `fret-dnd` extraction is evidence-driven rather than speculative.

Current landed outcome:

- the first stable contract is a vertical row/list/outliner recipe,
- `ecosystem/fret-ui-kit::recipes::imui_sortable` is the public owner,
- and extraction into `fret-dnd` is explicitly deferred pending a second real consumer.

## Phase C - Proof-first implementation

Status: Completed

Goal:

- land the recipe where it actually reduces authoring noise.

Deliverables:

- one reusable recipe in `ecosystem/fret-ui-kit::recipes`,
- optional pure/data-only helper extraction into `ecosystem/fret-dnd` if justified,
- migrated reorderable outliner proof in `apps/fret-examples/src/imui_editor_proof_demo.rs`.

Current landed slice:

- `ecosystem/fret-ui-kit::recipes::imui_sortable` now exposes `sortable_row(...)`,
  `sortable_row_with_options(...)`, `SortableInsertionSide`, and `reorder_vec_by_key(...)`,
- `apps/fret-examples/src/imui_editor_proof_demo.rs` now uses the recipe for the reorderable
  outliner proof,
- and `ecosystem/fret-imui/src/tests/interaction.rs` now exercises the same recipe in the real
  pointer interaction gate.

Current decision:

- the workstream now explicitly treats those two surfaces as sufficient second-surface evidence for
  the current narrow vertical-list contract,
- so no extra first-party demo is required before v1 closeout unless the recipe contract widens.

Exit gates:

- the proof/demo surface is materially simpler than the current app-local reorder packaging,
- the asset-slot proof still demonstrates the raw typed seam without recipe leakage,
- and `fret-ui-kit::imui` did not absorb reorder policy.

## Phase D - Gates and closeout

Status: Completed

Goal:

- leave a durable regression/evidence package and decide what remains after the first recipe pass.

Deliverables:

- focused recipe/unit tests,
- a real interaction gate for a recipe-backed reorder flow,
- closeout notes with the surviving defer list and owner handoff.

Current landed slice:

- recipe/unit coverage now lives in `ecosystem/fret-ui-kit/src/recipes/imui_sortable.rs`,
- compile-surface smoke now lives in `ecosystem/fret-ui-kit/tests/imui_sortable_recipe_smoke.rs`,
- and the existing `imui` reorder interaction gate remains green after the migration to the recipe.

Closeout note:

- the recipe contract is now considered closed for the current single-list vertical-row scope,
- no extra first-party demo is required before closeout,
- and wider reorder problems are deferred as new-contract lanes rather than carried as unfinished
  work here.

Exit gates:

- the recipe contract is reviewable through proof + gate + docs,
- the remaining follow-ons, if any, are short and explicitly owned,
- and the lane can close without reopening `imui` helper sprawl.
