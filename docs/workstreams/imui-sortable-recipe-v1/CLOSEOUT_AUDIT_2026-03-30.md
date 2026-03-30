# Closeout Audit — 2026-03-30

This audit records the final closeout read for the `imui-sortable-recipe-v1` lane.

Goal:

- verify what the first immediate sortable recipe actually closed,
- separate the landed vertical row recipe from the explicitly deferred follow-on contracts,
- and decide whether this lane should remain active or become historical closeout evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/imui-sortable-recipe-v1/DESIGN.md`
- `docs/workstreams/imui-sortable-recipe-v1/TODO.md`
- `docs/workstreams/imui-sortable-recipe-v1/MILESTONES.md`
- `docs/workstreams/imui-sortable-recipe-v1/SECOND_PROOF_SURFACE_DECISION_2026-03-30.md`

Implementation / proof anchors:

- `ecosystem/fret-ui-kit/src/recipes/imui_sortable.rs`
- `ecosystem/fret-ui-kit/src/recipes/mod.rs`
- `ecosystem/fret-ui-kit/tests/imui_sortable_recipe_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

Validation run used for closeout:

- `cargo test -p fret-ui-kit --features imui imui_sortable`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_sortable_recipe_smoke --test imui_drag_drop_smoke`
- `cargo test -p fret-imui sortable_rows_reorder_using_drop_positions`
- `cargo test -p fret-imui drag_drop_helper_previews_and_delivers_payload`
- `cargo test -p fret-examples --lib proof_outliner_reorder_moves_item_after_target`
- `cargo test -p fret-examples --lib proof_outliner_reorder_moves_item_before_target`
- `cargo check -p fret-examples --lib`

## Findings

### 1. The v1 recipe question is now closed on the intended owner layer

This lane opened to answer one narrow question:

> can Fret ship a reusable immediate sortable/reorder recipe without pushing sortable policy back
> into `fret-ui-kit::imui`?

The current repo now answers that question with a shipped yes.

The landed public recipe surface is:

- `sortable_row(...)`
- `sortable_row_with_options(...)`
- `SortableRowResponse::{preview_reorder, delivered_reorder}`
- `SortableInsertionSide`
- `reorder_vec_by_key(...)`

Most importantly, that surface lives in the intended owner:

- `ecosystem/fret-ui-kit::recipes::imui_sortable`

Conclusion:

- the ownership question for the first immediate sortable recipe is closed,
- and the correct owner remains `ecosystem/fret-ui-kit::recipes`, not `fret-ui-kit::imui`.

### 2. The recipe meaningfully reduces authoring noise on real immediate surfaces

Before this lane, reorder authoring pressure still repeated the same local steps:

- source/target drag wiring,
- vertical midpoint insertion-side derivation,
- preview/delivery packaging,
- and list mutation glue.

The current landed slice removes that repetition from two different immediate surfaces:

1. the app-facing tree/outliner proof in
   `apps/fret-examples/src/imui_editor_proof_demo.rs`
2. the flat-row real pointer interaction gate in
   `ecosystem/fret-imui/src/tests/interaction.rs`

That matters because the recipe is no longer only a toy helper hidden in one demo.
It now proves itself on:

- a hierarchical tree-node style authoring surface,
- and a flat button-row interaction surface.

Conclusion:

- the recipe clears the “real reduction” bar for the current v1 scope.

### 3. The lower `imui` seam stayed clean, and that is the main architectural success

The most important negative result is what did not happen.

This lane did **not**:

- add sortable insertion policy to `fret-ui-kit::imui`,
- widen the runtime drag contract,
- or force an extraction into `fret-dnd` just to make the recipe look more abstract.

Instead:

- the raw typed drag/drop seam remains where it belongs,
- the asset-slot proof still uses that seam directly,
- and the new sortable recipe sits one layer above it.

This is the architectural bar the lane needed to hit.

Conclusion:

- the mechanism vs recipe split held,
- and that means the lane is successful even before considering future follow-ons.

### 4. Extraction into `fret-dnd` is still not justified

The lane also had to answer a subtler question:

> should insertion-side math or reorder packaging be extracted into `fret-dnd` immediately?

Current answer:

- no.

Why:

- the current helper is still tightly shaped around immediate row integration,
- the evidence set still revolves around one contract family (single vertical list reorder),
- and there is not yet a second distinct shared consumer that proves a pure/data-only extraction is
  worth carrying as a stable lower-level contract.

Conclusion:

- keep the helper recipe-local for now,
- and reopen `fret-dnd` extraction only with fresh cross-consumer evidence.

### 5. The remaining backlog is a new-contract backlog, not an unfinished v1 backlog

What remains after this closeout is real, but it is not “finish v1.”

The surviving deferred items are:

- source ghost / drag preview chrome,
- auto-scroll during drag,
- multi-container sortable transfer,
- richer collision/placement policy,
- docking/workspace shell-specific reorder choreography.

Those are not just “more examples” of the current contract.
They are new contract questions.

Conclusion:

- this folder should now be read as closeout evidence for the first vertical row recipe,
- and any further work should open or extend a new lane only when one of those deferred contracts
  is intentionally taken on.

## Decision from this audit

Treat `imui-sortable-recipe-v1` as:

- closed for the first immediate sortable recipe question,
- historical closeout evidence by default,
- and reopenable only if the repo intentionally takes on a wider reorder contract.

## Immediate execution consequence

From this point forward:

1. keep `sortable_row(...)` as the shipped v1 row-level recipe boundary,
2. keep `fret-ui-kit::imui` limited to the typed drag/drop seam plus geometry signals,
3. keep `reorder_vec_by_key(...)` and insertion-side packaging in the recipe layer,
4. do not extract new `fret-dnd` helpers without a second distinct shared consumer,
5. treat source ghost, auto-scroll, multi-container transfer, and shell-specific reorder behavior
   as future contract lanes rather than unfinished v1 tasks.
