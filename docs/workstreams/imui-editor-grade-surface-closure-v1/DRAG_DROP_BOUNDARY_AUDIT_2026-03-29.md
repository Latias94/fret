# imui drag/drop boundary audit — 2026-03-29

Status: landed M3 slice
Last updated: 2026-03-29

Related:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/TODO.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/MILESTONES.md`
- `crates/fret-runtime/src/ui_host.rs`
- `crates/fret-ui/src/action.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Question

Can `imui` ship a clean immediate drag/drop payload surface without widening the runtime contract or
adding a stringly compatibility layer?

## Short answer

Yes, but not by writing typed payloads directly into `UiDragActionHost`.

The shipped seam is:

- keep the existing runtime `DragSession` contract unchanged,
- reuse the current pressable drag threshold and session routing,
- and publish typed payloads through a model-backed `imui` store keyed by `DragSessionId`.

This lands as a generic helper family in `fret-ui-kit::imui`:

- `drag_source(...)`
- `drag_source_with_options(...)`
- `drop_target::<T>(...)`
- `drop_target_with_options::<T>(...)`

## Why the direct runtime-payload route is not the right owner seam

Runtime facts:

- `DragHost` on `UiHost` can begin typed drag sessions (`begin_drag_with_kind<T>`,
  `begin_cross_window_drag_with_kind<T>`).
- But stored pointer hooks in `fret-ui` operate through the object-safe `UiDragActionHost`.
- `UiDragActionHost` intentionally begins runtime drag sessions with `()` payloads.

Implication:

- an `imui` helper that lives in pressable/pointer action hooks cannot portably create typed
  `DragSession` payloads today,
- and forcing generic payload APIs into the object-safe action host would widen a hard runtime
  seam for a facade-level authoring need.

That would be the wrong boundary move for this workstream.

## Shipped design

### 1. Source and target stay response-driven

The helper does not invent a parallel wrapper element grammar.

Instead, call sites:

- render an existing pressable-style item and get `ResponseExt`,
- attach `drag_source(...)` to that response,
- attach `drop_target::<T>(...)` to another response.

This matches the already-shipped tooltip pattern better than cloning Dear ImGui's
`BeginDragDropSource/Target` block grammar.

### 2. Payloads live in an immediate store, not in `DragSession`

`fret-ui-kit::imui` now keeps a model-backed store that records:

- active typed payloads by `DragSessionId`,
- source `GlobalElementId`,
- active/delivered pointer positions for upper-layer policy derivation,
- and one-frame delivery records for typed drop targets.

This store is populated from pointer hooks and read back during the next render.

### 3. Existing drag mechanics are reused

The helper reuses the current pressable drag lifecycle:

- pointer-down starts the runtime drag session,
- pointer-move crosses the existing threshold,
- source publication happens once the runtime session is actually dragging,
- target preview uses the current raw hover signal,
- target delivery is latched on pointer-up and surfaced once on the next render.

Implementation note:

- delivery is intentionally finalized from the pressed source side rather than from the hovered
  target side,
- because `pressable` pointer-up routing is owned by the pressed source gesture, not by whichever
  target currently sits under the pointer,
- so the helper tracks the currently hovered target in the `imui` drag/drop store and resolves the
  drop from source-side pointer-up.

### 4. Cross-window is opt-in, not a second stack

`DragSourceOptions::cross_window` upgrades the trigger's session to
`begin_cross_window_drag_with_kind(...)`.

This reuses the runtime's existing multi-window hover/drop routing instead of creating a separate
immediate drag stack.

## What this closes

This closes the missing **typed immediate source/target payload authoring seam** for common editor
workflows such as:

- asset chips dropped onto property slots,
- tool-panel item transfer,
- lightweight outliner/item target handoff when collision math is not required.

It also now exposes enough geometry context for upper layers to derive sortable insertion policies
without widening the runtime contract:

- preview pointer position on an active compatible target,
- delivered pointer position on the next render after drop.

## What this intentionally does not close

This slice does not try to solve:

- sortable insertion math,
- arbitrary rect collision registries,
- auto-scroll during drag,
- docking/workspace shell choreography,
- OS/native external drag-and-drop,
- or preview-tooltip chrome.

The first reorderable outliner proof now computes insertion side at the app layer from
`DropTargetResponse::{preview_position, delivered_position}` rather than from an `imui`-owned
sortable policy helper.

Those remain correctly owned by:

- `fret-ui-kit::dnd`,
- `fret-dnd`,
- recipes,
- docking/workspace crates,
- or future external/platform integration work.

## Proof and gates

Proof surface:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
  now includes a typed asset-chip to material-slot drag/drop slice plus a reorderable outliner
  proof that keeps sortable math app-owned.

Focused gates:

- `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs`
- `cargo test -p fret-ui-kit --features imui drag_drop --lib`
- `cargo test -p fret-imui drag_drop_helper_previews_and_delivers_payload`
- `cargo test -p fret-imui sortable_rows_reorder_using_drop_positions`

Implementation anchors:

- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

## Maintainer conclusion

The correct M3 move was **not** to widen the runtime action-host contract.

The correct move was to land a thin, typed, response-driven facade seam in
`fret-ui-kit::imui`, backed by the already-existing runtime drag session lifecycle.
