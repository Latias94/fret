# Closeout Audit — 2026-03-30

This audit records the final closeout read for the `imui-cross-window-ghost-v1` lane.

Goal:

- verify what the first generic cross-window drag-preview lane actually closed,
- separate the landed `current_window` transfer baseline from the explicitly deferred shell-aware
  choreography,
- and decide whether this lane should remain active or become historical closeout evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`
- `docs/workstreams/imui-cross-window-ghost-v1/TODO.md`
- `docs/workstreams/imui-cross-window-ghost-v1/MILESTONES.md`
- `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Implementation / proof anchors:

- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs`
- `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_drag_preview_smoke.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Validation run used for closeout:

- `cargo test -p fret-ui-kit --features imui --test imui_drag_drop_smoke --test imui_drag_preview_smoke`
- `cargo test -p fret-ui-kit --features imui ghost_anchor_`
- `cargo test -p fret-imui tests::interaction::cross_window_drag_preview_ghost_transfers_between_windows -- --exact`
- `cargo check -p fret-examples`
- `python tools/check_layering.py`

## Findings

### 1. The generic cross-window ownership question is now closed on the intended owner layer

This lane opened to answer one narrow question:

> can Fret ship one coherent cross-window ghost baseline without widening runtime into a preview
> policy registry and without pushing shell choreography down into `imui`?

The current repo now answers that question with a shipped yes.

The landed generic recipe surface is:

- `publish_cross_window_drag_preview_ghost(...)`
- `publish_cross_window_drag_preview_ghost_with_options(...)`
- `render_cross_window_drag_preview_ghosts(...)`

Most importantly, that surface lives in the intended owner:

- `ecosystem/fret-ui-kit::recipes::imui_drag_preview`

Conclusion:

- the ownership question for the generic cross-window transfer helper is closed,
- and the correct owner remains `ecosystem/fret-ui-kit::recipes`, not `fret-ui-kit::imui`.

### 2. The mechanism delta stayed observational, which is the main contract success

The lower seam changed only enough to let recipe-owned transfer stay keyed to real drag sessions:

- `DragSourceResponse` now exposes `pointer_id()` and `session_id()`,
- and `drag_drop.rs` fills those accessors from the live runtime drag session.

What did **not** happen matters more:

- no preview builder was added to `drag_source(...)`,
- no runtime preview registry was introduced,
- and `fret-ui-kit::imui` still does not own shell arbitration logic.

Conclusion:

- the mechanism layer stayed observational,
- and the lane succeeded because it did not widen the runtime/helper contract in the wrong place.

### 3. The shipped transfer rule is now explicit and reviewable

The landed transfer baseline is:

- the source still authors preview meaning,
- the recipe layer publishes a descriptor keyed by `DragSessionId`,
- and only `drag.current_window` paints the ghost in a participating window root.

This matters because the repo can now answer the previously open generic questions concretely:

- duplicate ghosts are prevented by single-window paint ownership,
- source meaning survives transfer without a global style registry,
- and the same-window helper remains intact for environments or authoring sites that do not need
  cross-window choreography.

Conclusion:

- the generic `current_window` transfer contract is now closed and auditable.

### 4. Cross-window close synchronization needed one extra frame, not a wider runtime

The most important implementation surprise was not descriptor publication.
It was teardown synchronization.

Without an explicit stale-retention step, the first window to notice a finished drag could prune the
descriptor before a second participating window had a chance to observe `open=false`.

The landed fix stays narrow and recipe-owned:

- stale descriptors are marked with `stale_frame`,
- all participating windows can still sync closed overlays during that frame,
- and cleanup happens on the next frame.

Conclusion:

- the real complexity was close synchronization across window roots,
- and it was solved without widening runtime contracts.

### 5. The first proof/gate package is sufficient for the generic slice

This lane is no longer justified only by theory.

The landed proof package now covers:

1. a first-party multi-window proof surface in
   `apps/fret-examples/src/imui_editor_proof_demo.rs`
2. compile-surface coverage for the new recipe and response accessors in
   `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs` and
   `ecosystem/fret-ui-kit/tests/imui_drag_preview_smoke.rs`
3. a real ownership-transfer interaction gate in
   `ecosystem/fret-imui/src/tests/interaction.rs`

That proof matters because the repo now has direct evidence for:

- source-window visible ghost,
- hovered-window ghost handoff,
- no-duplicate visibility across participating windows,
- and release/cancel cleanup.

Conclusion:

- the lane cleared the “proof + gate + evidence” bar for the generic cross-window slice.

### 6. The remaining backlog is a shell-aware backlog, not unfinished generic work

What remains after this lane is real, but it is not “finish the same helper.”

The surviving deferred items are:

- docking/tear-out ghost choreography,
- workspace/viewport shell arbitration,
- aggregate previews,
- native/external preview surfaces,
- richer descriptor transport only if shell-aware proof forces it.

Those are new owner questions.
They are not missing pieces of the generic `current_window` transfer baseline.

Conclusion:

- this folder should now be read as closeout evidence for the generic cross-window ghost contract,
- and the next meaningful step should open a new shell-aware lane rather than keep this one
  nominally active.

## Decision from this audit

Treat `imui-cross-window-ghost-v1` as:

- closed for the generic cross-window transfer baseline,
- historical closeout evidence by default,
- and succeeded by `docs/workstreams/imui-shell-ghost-choreography-v1/` for the next shell-aware
  choreography lane.

## Immediate execution consequence

From this point forward:

1. keep `DragSourceResponse::{pointer_id, session_id}` as observation-only support seams,
2. keep generic transfer publication/rendering in `ecosystem/fret-ui-kit::recipes`,
3. keep `render_cross_window_drag_preview_ghosts(...)` a once-per-window-root call site,
4. keep shell-aware docking/workspace ghost ownership out of `fret-ui-kit::imui`,
5. move docking/tear-out choreography and wider shell arbitration into the successor lane.
