# Closeout Audit — 2026-03-30

This audit records the final closeout read for the `imui-drag-preview-ghost-v1` lane.

Goal:

- verify what the first source-side drag preview lane actually closed,
- separate the landed same-window ghost from the explicitly deferred cross-window choreography,
- and decide whether this lane should remain active or become historical closeout evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/imui-drag-preview-ghost-v1/DESIGN.md`
- `docs/workstreams/imui-drag-preview-ghost-v1/TODO.md`
- `docs/workstreams/imui-drag-preview-ghost-v1/MILESTONES.md`
- `docs/workstreams/imui-drag-preview-ghost-v1/UPSTREAM_PARITY_AUDIT_2026-03-30.md`

Implementation / proof anchors:

- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs`
- `ecosystem/fret-ui-kit/src/recipes/mod.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/tests/pointer_capture.rs`
- `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_drag_preview_smoke.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Validation run used for closeout:

- `cargo test -p fret-ui-kit --features imui --test imui_drag_drop_smoke --test imui_drag_preview_smoke`
- `cargo test -p fret-ui-kit --features imui imui_drag_preview`
- `cargo test -p fret-ui-kit pointer_capture_keeps_noninteractive_hover_overlays_and_tooltips_visible`
- `cargo test -p fret-imui drag_preview_ghost_follows_pointer_and_clears_on_release`
- `cargo test -p fret-imui drag_drop_helper_previews_and_delivers_payload`
- `cargo test -p fret-imui sortable_rows_reorder_using_drop_positions`
- `cargo check -p fret-examples --lib`

## Findings

### 1. The owner question is now closed on the intended layer

This lane opened to answer one narrow question:

> can Fret ship a real source-side drag preview ghost without widening runtime contracts or turning
> `fret-ui-kit::imui` into a preview-policy layer?

The current repo now answers that question with a shipped yes.

The landed public ghost surface is:

- `DragPreviewGhostOptions`
- `drag_preview_ghost(...)`
- `drag_preview_ghost_with_options(...)`

Most importantly, that surface lives in the intended owner:

- `ecosystem/fret-ui-kit::recipes::imui_drag_preview`

Conclusion:

- the ownership question for the first public drag preview helper is closed,
- and the correct owner remains `ecosystem/fret-ui-kit::recipes`, not `fret-ui-kit::imui`.

### 2. The mechanism delta stayed minimal, which is the main contract success

The mechanism-side addition is intentionally small:

- `DragSourceResponse::position()` now exposes the current drag pointer position,
- and no preview-builder closures, preview policies, or style defaults were added to
  `drag_source(...)` or `DragSourceOptions`.

This is the exact support seam the lane wanted:

- observational,
- source-local,
- and sufficient for source-authored preview content.

Conclusion:

- the `imui` seam stayed clean,
- and the lane succeeded precisely because it did not widen the runtime or helper contract surface.

### 3. The first v1 ghost needed one overlay substrate adjustment, but not a new runtime

The only non-trivial implementation surprise was overlay arbitration during pointer capture.

Without an adjustment, source-side drag ghosts disappeared while the source layer owned capture,
because tooltip/hover overlay visibility was being suppressed whenever another layer held capture.

The landed fix was still small and mechanism-appropriate:

- non-interactive hover/tooltip overlays remain visible during capture,
- while interactive hover/tooltip overlays keep their prior suppression behavior.

This matters because it preserves the intended architecture:

- no second drag runtime,
- no preview-specific runtime host API,
- and no special-case escape hatch in `imui`.

Conclusion:

- the overlay substrate was sufficient after a narrow visibility/arbitration correction.

### 4. The helper now proves itself on two real immediate authoring surfaces

The lane is no longer justified only by theory.

The landed proof package now covers:

1. asset-chip to material-slot drag in
   `apps/fret-examples/src/imui_editor_proof_demo.rs`
2. row-level immediate drag interaction in
   `ecosystem/fret-imui/src/tests/interaction.rs`

That proof matters because the helper now demonstrates:

- non-sortable payload drags,
- sortable-row composition,
- same-window pointer following,
- and release cleanup.

Conclusion:

- the helper cleared the “first-party proof surface” bar for the v1 same-window slice.

### 5. The remaining backlog is a new-contract backlog, not unfinished same-window v1 work

What remains after this lane is real, but it is not “finish the same helper.”

The surviving deferred items are:

- cross-window ghost ownership and visibility transfer,
- docking/workspace shell choreography,
- multi-item aggregate preview,
- native/external preview surfaces.

Those are new contract questions, not minor follow-up polish.

Conclusion:

- this folder should now be read as closeout evidence for the same-window source-side ghost,
- and the next meaningful step should open a new lane rather than keep this one nominally active.

## Decision from this audit

Treat `imui-drag-preview-ghost-v1` as:

- closed for the same-window source-side drag preview question,
- historical closeout evidence by default,
- and succeeded by `docs/workstreams/imui-cross-window-ghost-v1/` for the next choreography lane.

## Immediate execution consequence

From this point forward:

1. keep `DragSourceResponse::position()` as the only `imui` support seam added by this lane,
2. keep `drag_preview_ghost(...)` and `drag_preview_ghost_with_options(...)` recipe-owned,
3. keep preview content source-authored through `IntoUiElement`,
4. treat same-window ghost behavior as shipped and closed,
5. move cross-window ownership, shell choreography, and wider preview transfer questions into
   `imui-cross-window-ghost-v1`.
