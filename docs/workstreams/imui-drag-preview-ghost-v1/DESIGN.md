# imui drag preview ghost v1 - design

Status: active workstream

Last updated: 2026-03-30

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/imui-drag-preview-ghost-v1/UPSTREAM_PARITY_AUDIT_2026-03-30.md`

## Purpose

This workstream is the next immediate follow-on after two shipped lanes:

- the typed `drag_source(...)` / `drop_target::<T>(...)` seam in `fret-ui-kit::imui`
- the first vertical row reorder recipe in `fret-ui-kit::recipes::imui_sortable`

Those lanes closed payload publication, target-side preview geometry, delivery, and row-level
reorder packaging.

They did **not** close source-side drag preview chrome.

This lane exists to answer one focused question:

> how should Fret ship a real immediate drag preview ghost without widening the runtime contract or
> turning `fret-ui-kit::imui` into a preview-policy layer?

## Current assessment

Fret is now in a very specific state:

- typed drag payloads are already shipped,
- compatible targets already receive preview and delivery geometry,
- the current sortable recipe already proves higher-level policy can stay above the `imui` seam,
- and the repo already has overlay/floating infrastructure for click-through absolute panels.

What is still missing is not another payload seam.
What is still missing is a **source-side visual contract**.

Compared with upstream references:

- Dear ImGui treats drag preview as part of source authoring: `BeginDragDropSource()` opens a
  preview tooltip by default and the source renders its own preview content.
- egui treats drag preview as part of source painting: `Ui::dnd_drag_source(...)` paints the source
  body on a tooltip-order layer and translates that layer to the pointer.

Fret does not yet expose the equivalent outcome.

Today the public `imui` seam stops at:

- `DragSourceResponse { active, cross_window }`
- `DropTargetResponse::{preview_position, delivered_position}`

So the current gap is narrow and concrete:

- there is no public drag-position readout on the source response,
- there is no immediate helper that paints a click-through preview overlay near the pointer,
- and there is no recipe-level ghost surface that first-party examples can teach.

## Why this lane should exist

Without an explicit workstream, source preview will likely drift into one of three wrong outcomes:

- app-local ad hoc overlay code inside proof demos,
- preview-policy knobs added directly to `drag_source(...)`,
- or a larger runtime widening pass that is not justified by the actual gap.

This lane exists to prevent all three.

The repo already proved the right architectural split:

- runtime owns drag sessions,
- `imui` owns the typed source/target seam plus geometry signals,
- recipes own reusable higher-level policy packaging.

The next move should keep that split intact.

## Goals

### G1 - Keep the owner split explicit

This workstream must preserve the following rule:

- `fret-ui-kit::imui` may grow only the **smallest read-only drag observation seam** needed by
  higher layers,
- `fret-ui-kit::recipes` should own the first reusable drag preview ghost helper,
- product/demo crates should keep ownership of preview content and domain semantics,
- and `crates/fret-ui` / `fret-runtime` should remain unchanged unless a truly generic mechanism
  gap is proven.

### G2 - Freeze the first slice to same-window source preview

The first stable contract should solve one narrow problem well:

- a source-side preview ghost that follows the pointer in the current window,
- stays non-interactive / click-through,
- composes with the already-shipped typed drag seam,
- and is teachable on the first-party proof surfaces.

This first slice should not try to solve all preview choreography at once.

### G3 - Keep preview authored at the source

The preview ghost should be authored where the source already knows:

- the payload meaning,
- the visual it wants to show,
- and the current source-local state.

That means the first Fret contract should lean toward source-side preview authoring, not toward
target-side preview hooks or a global payload-lookup API.

### G4 - Preserve the fearless refactor posture

This lane explicitly allows a flag-day cleanup.

If a first prototype proves the API shape is wrong:

- delete it,
- rename it,
- or move it to the correct owner,

instead of carrying aliases or compatibility helpers.

## Non-goals

- Cross-window preview ghost choreography in v1.
- Multi-item aggregate ghost composition in v1.
- Docking/workspace shell tear-off preview choreography.
- OS/native external drag-and-drop preview surfaces.
- Adding preview-builder closures or preview-style matrices directly to `DragSourceOptions`.
- Widening the object-safe runtime action-host contract for preview-specific needs.
- Reopening sortable/reorder policy inside this lane.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `crates/fret-ui` / `fret-runtime` | drag session lifecycle, overlay-root mechanisms, pointer routing | ghost chrome defaults, source-preview authoring policy |
| `ecosystem/fret-ui-kit::imui` | typed drag/drop seam, drag-session observation signals, small reusable overlay glue if needed | ghost styling defaults, product preview recipes, sortable policy |
| `ecosystem/fret-ui-kit::recipes` | reusable source-side drag preview ghost helper and first stable options | runtime drag ownership, docking/workspace shell choreography |
| `apps/fret-examples` and product crates | preview content, domain labels/icons/counts, product-specific semantics | pretending product-local visuals are a generic lower-level mechanism |

## Decision snapshot

### 1) The first public ghost surface should live in `fret-ui-kit::recipes`

The preview ghost is not another mechanism primitive.
It is higher-level authoring policy:

- follow the pointer,
- paint above content,
- ignore pointer hit-testing,
- and apply stable default offset/chrome semantics.

That makes `fret-ui-kit::recipes` the right first public owner.

### 2) `imui` may need a tiny support seam, but only a geometry seam

The current preferred support delta is small and read-only:

- expose the active drag pointer position on `DragSourceResponse`,
- optionally also expose drag start position if the prototype proves it useful,
- and keep that seam purely observational.

Current preference:

- extend `DragSourceResponse` rather than adding a new global payload query API,
- because the source call site already owns the payload and can author the preview content locally.

### 3) The first slice should reuse existing overlay infrastructure

The ghost should not create a second drag runtime.

The intended implementation direction is:

- reuse the current overlay/request machinery already used by `imui::tooltip_overlay`,
- but drive visibility from active drag state instead of hover/focus state,
- and render a non-interactive absolute panel near the drag pointer.

Implementation note after the first landing:

- non-interactive hover/tooltip overlays must remain visible during pointer capture,
  otherwise source-side drag ghosts disappear while the source layer owns capture.
  This was resolved inside `window_overlays::render` without widening runtime contracts.

### 4) Cross-window ghost is explicitly deferred

Cross-window drag routing already exists, but cross-window preview ownership is a different
contract:

- which window paints the ghost,
- whether the source window hides its ghost once hover moves out,
- how duplicate ghosts are avoided,
- and how viewport/window overlays arbitrate preview z-order.

That is too much policy for the first slice.

### 5) Sortable and ghost remain separate follow-ons

The current sortable recipe lane is closed.
This lane must compose with it, not absorb it.

That means:

- row reorder continues to use `sortable_row(...)`,
- insertion-line math remains recipe-owned in the sortable lane,
- and the ghost helper should stay reusable for both sortable rows and non-sortable asset/item
  drags.

## Target architecture

### `ecosystem/fret-ui-kit::imui`

Expected minimal delta:

- `DragSourceResponse` grows drag-position visibility,
- internal implementation keeps sourcing that position from the already-existing runtime drag
  session,
- and no preview styling or source-preview builder closure lands on the public `drag_source(...)`
  family itself.

Representative support surface:

- `DragSourceResponse::{position, start_position}` or equivalent getter methods,
- optional small internal overlay bridge reused by recipes.

### `ecosystem/fret-ui-kit::recipes`

Expected first public owner for the reusable preview surface.

Representative target:

- `imui_drag_preview` or equivalent recipe module,
- the landed module is `recipes::imui_drag_preview`,
- one canonical helper family such as `drag_preview_ghost(...)` /
  `drag_preview_ghost_with_options(...)`,
- default click-through overlay behavior,
- and source-authored preview content passed as `IntoUiElement`.

The exact final names are less important than the rule:

- the first public ghost contract should read like a recipe,
- not like a new low-level `imui` runtime surface.

### Proof surfaces

The first slice should prove itself on two already-relevant surfaces:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

Preferred proof scenarios:

- the asset-chip to material-slot drag uses a small chip/card ghost,
- the reorderable outliner uses a row ghost while keeping insertion-line logic separate.

## Regression and proof requirements

Minimum proof/gate package expected for implementation:

- one focused recipe/unit test locking the preview helper contract,
- one compile-surface smoke test for the new recipe module,
- one real pointer interaction test proving the ghost tracks drag activity,
- and one first-party proof/demo update showing the source-authored preview on a real surface.

The lower typed drag/drop seam must stay green while this lands:

- `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs` typed payload gates

## Success criteria

This lane is successful when:

- Fret ships a teachable same-window source preview ghost surface,
- the public preview helper clearly lives above `fret-ui-kit::imui`,
- `DragSourceResponse` is still a small geometry/status readout rather than a policy object,
- the first-party proof surface becomes visibly closer to ImGui/egui drag authoring outcomes,
- and cross-window / multi-item / shell-specific preview behavior remains short, explicit, and
  deferred.
