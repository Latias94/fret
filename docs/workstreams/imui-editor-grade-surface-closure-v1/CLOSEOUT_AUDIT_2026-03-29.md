# Closeout Audit — 2026-03-29

This audit records the final closeout read for the `imui-editor-grade-surface-closure-v1` lane.

Goal:

- verify which editor-grade `imui` gaps were actually closed,
- separate the shipped helper closure from the intentionally deferred sortable/reorder policy work,
- and decide whether this lane should remain active or become historical closeout evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/TODO.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/MILESTONES.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/EDITOR_GRADE_GAP_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`

Implementation / proof anchors:

- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs`

Validation run used for closeout:

- `cargo test -p fret-imui drag_drop_helper_previews_and_delivers_payload`
- `cargo test -p fret-imui sortable_rows_reorder_using_drop_positions`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_drag_drop_smoke`
- `cargo check -p fret-examples --lib`
- `cargo test -p fret-examples --lib proof_outliner_reorder_moves_item_after_target`

## Findings

### 1. The editor composite adapter gap is now closed on the intended owner layer

This lane no longer has an active backlog for the core editor inspector skeleton surfaces.

The shipped surface now includes thin immediate adapters for:

- `PropertyGroup`
- `PropertyGrid`
- `PropertyGridVirtualized`
- `InspectorPanel`

This matters because the original gap was not primitive scarcity.
The real gap was that real inspector/outliner screens still had to fall back to noisier
declarative wrapping for common editor composites.

The current proof surface now demonstrates that those composites can be authored directly from the
immediate layer:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`

Conclusion:

- the composite-closure goal for this lane is shipped,
- and the correct owner remains `ecosystem/fret-ui-editor::imui`.

### 2. Tooltip and tree/disclosure authoring are now first-class generic helpers

The generic immediate helper closure also landed on the intended owner layer.

`fret-ui-kit::imui` now exposes first-class tooltip and disclosure/tree helpers instead of forcing
each call site to wire hover/disclosure behavior ad hoc.

The proof/demo surface now contains:

- tooltip usage on the editor proof controls,
- an immediate outliner slice with explicit stable ids and explicit hierarchy depth,
- and composite-heavy inspector content that no longer looks like missing helper coverage.

Conclusion:

- the remaining generic helper gap is no longer tooltip/tree/disclosure,
- `fret-imui` stayed minimal,
- and `fret-ui-kit::imui` absorbed the right generic policy glue without becoming a shell-policy
  crate.

### 3. The typed drag/drop seam is closed, but sortable policy was intentionally not pulled into `imui`

The drag/drop audit and implementation now answer the main M3 question cleanly:

- typed payload authoring is shipped,
- the runtime action-host contract was not widened,
- and the helper remains response-driven and model-backed.

The most important closeout detail is what did **not** happen:

- no sortable insertion policy landed in `fret-ui-kit::imui`,
- no reorder-specific API was added to the immediate helper layer,
- and no stringly compatibility stopgap was introduced.

Instead, the shipped seam now provides:

- `drag_source(...)` / `drag_source_with_options(...)`,
- `drop_target::<T>(...)` / `drop_target_with_options::<T>(...)`,
- `DropTargetResponse::{preview_position, delivered_position}`,
- and a proof that app-owned reorder math can be built on top of those signals.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

Conclusion:

- the missing typed payload seam is closed,
- but reusable sortable/reorder policy remains deliberately outside this lane.

### 4. The surviving backlog belongs to recipe/policy layers, not to more `imui` helpers

After the landed closure pass, the remaining pressure relative to Dear ImGui and egui is no longer
"we need more immediate primitives."

The surviving reusable gap is:

- a teachable, reusable sortable/reorder recipe for immediate rows/lists/outliners.

That gap should now be owned by:

- `ecosystem/fret-ui-kit::recipes` as the default first owner,
- and `ecosystem/fret-dnd` only if a small pure/data-only insertion helper becomes clearly shared.

It should **not** be owned by:

- `ecosystem/fret-ui-kit::imui`,
- `ecosystem/fret-imui`,
- or a new runtime contract widening pass.

Still-deferred items remain out of scope for this lane:

- source ghost / preview chrome,
- auto-scroll during drag,
- multi-container sortable choreography,
- docking/workspace-specific shell reordering,
- external/native drag-and-drop.

Conclusion:

- this folder should now be read as closeout evidence,
- and the active next lane should be a sortable recipe workstream rather than another helper-growth
  pass in `imui`.

## Decision from this audit

Treat `imui-editor-grade-surface-closure-v1` as:

- closed for the current editor-grade helper closure question,
- historical closeout evidence by default,
- and superseded for active follow-on work by `docs/workstreams/imui-sortable-recipe-v1/`.

## Immediate execution consequence

From this point forward:

1. keep `fret-ui-kit::imui` limited to the typed drag/drop seam plus generic geometry signals,
2. do not add sortable/reorder policy helpers to `imui`,
3. land reusable reorder flows in `ecosystem/fret-ui-kit::recipes`,
4. extract helper logic into `ecosystem/fret-dnd` only if it is clearly shared and pure/data-only,
5. keep `apps/fret-examples/src/imui_editor_proof_demo.rs` plus
   `ecosystem/fret-imui/src/tests/interaction.rs` as the minimum proof/gate package for the next
   recipe lane.
