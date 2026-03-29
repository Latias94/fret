# imui sortable recipe v1 - TODO

Status: active planning board

Last updated: 2026-03-29

Tracking doc: `docs/workstreams/imui-sortable-recipe-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-sortable-recipe-v1/MILESTONES.md`

Predecessor closeout:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`

Related DnD baseline:

- `docs/workstreams/headless-dnd-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/headless-dnd-fearless-refactor-v1/TODO.md`

This board assumes a fearless refactor posture.
Compatibility shims are explicitly out of scope.

## M0 - Workstream setup and owner freeze

- [x] Create the workstream directory and initial design/TODO/milestones pack.
- [x] Record the owner split:
      `fret-ui-kit::imui` keeps the typed drag/drop seam;
      `fret-ui-kit::recipes` owns reusable sortable policy;
      `fret-dnd` only gains pure/data-only helpers if real sharing is proven.
- [x] Freeze the non-goals:
      no sortable helper growth in `imui`,
      no runtime contract widening,
      no compatibility aliases.
- [x] Link this workstream from `docs/workstreams/README.md`.

## M1 - Freeze the first stable recipe contract

- [ ] Decide the first stable target shape:
      single vertical list/outliner reorder before multi-container transfer.
- [ ] Define the minimum row integration surface for recipe adoption on an immediate list item.
- [ ] Decide whether insertion-side classification remains recipe-local or justifies extraction into
      `fret-dnd`.
- [ ] Freeze which responsibilities stay app-owned:
      item rendering, item identity, and final reorder mutation.
- [ ] Freeze which follow-on items stay explicitly out of scope for v1:
      source ghost, auto-scroll, multi-container transfer, docking/workspace shell choreography.

## M2 - Land the proof-first implementation

- [ ] Add the first reusable sortable recipe in `ecosystem/fret-ui-kit::recipes`.
- [ ] If justified, extract any shared pure/data-only helper into `ecosystem/fret-dnd`.
- [ ] Migrate the reorderable outliner proof in `apps/fret-examples/src/imui_editor_proof_demo.rs`
      to the recipe.
- [ ] Keep the asset-chip to material-slot proof on the raw drag/drop seam as boundary evidence.
- [ ] Decide whether a second proof surface is needed before the recipe contract is widened.

## M3 - Gates, docs, and closeout

- [ ] Add focused recipe tests that lock before/after insertion semantics.
- [ ] Upgrade or supplement the real interaction gate in
      `ecosystem/fret-imui/src/tests/interaction.rs` so a recipe-backed reorder flow is covered.
- [ ] Keep `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs` green to prove the lower seam did
      not absorb recipe policy.
- [ ] Capture a closeout summary that records:
      what the recipe closed,
      what stayed deferred,
      and whether any helper extraction into `fret-dnd` was actually justified.
