# imui drag preview ghost - upstream parity audit (2026-03-30)

Status: active audit note

Last updated: 2026-03-30

Related:

- `docs/workstreams/imui-drag-preview-ghost-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`

## Question

After shipping typed drag/drop and the first sortable recipe, what exactly is still missing in Fret
relative to Dear ImGui and egui for source-side drag preview behavior?

## Short answer

The missing gap is now narrow:

- Fret already has the payload seam,
- Fret already has target-side preview/delivery geometry,
- but Fret does not yet have a public source-side preview ghost outcome.

The correct next move is:

- keep runtime contracts unchanged,
- add at most a tiny read-only drag-position seam in `fret-ui-kit::imui`,
- and land the first preview ghost helper as a recipe-level surface.

## Upstream findings

### 1. Dear ImGui treats preview as part of source authoring

Relevant anchors:

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_demo.cpp`

Observed behavior:

- `BeginDragDropSource()` opens a preview tooltip by default.
- The source renders the preview content inline between `BeginDragDropSource()` and
  `EndDragDropSource()`.
- `ImGuiDragDropFlags_SourceNoPreviewTooltip` can suppress the default preview tooltip.
- `GetDragDropPayload()` allows source-side preview content to read the current payload directly.
- If the source stops submitting the source block, ImGui can still keep payload state alive, but the
  preview tooltip is no longer source-authored.

Architectural implication:

- ImGui keeps preview ownership close to the source.
- The preview surface is not target-owned and is not modeled as a separate global component API.

### 2. egui treats preview as a source-painted cursor layer

Relevant anchors:

- `repo-ref/egui/crates/egui/src/ui.rs`
- `repo-ref/egui/crates/egui/src/response.rs`
- `repo-ref/egui/crates/egui_demo_lib/src/demo/drag_and_drop.rs`

Observed behavior:

- `Ui::dnd_drag_source(...)` stores the payload while the source is being dragged.
- The same helper paints the source body onto a tooltip-order layer.
- egui then translates that painted layer to the current pointer position.
- Targets remain separate and only resolve hover/release payload semantics.

Architectural implication:

- egui also keeps the preview outcome source-owned.
- The target payload API and the source visual API stay separate but coordinated.

### 3. Current Fret is already aligned on payload ownership, but not on source preview outcome

Relevant anchors:

- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_surface.rs`

Current Fret behavior:

- `drag_source(...)` publishes typed payloads through a model-backed store keyed by `DragSessionId`.
- `drop_target::<T>(...)` resolves preview and delivery against compatible targets.
- `DropTargetResponse` already exposes `preview_position` and `delivered_position`.
- Overlay/floating infrastructure already exists for absolute, click-through, non-modal panels.

Current missing pieces:

- `DragSourceResponse` does not expose drag pointer geometry.
- There is no public helper that paints a pointer-following ghost during an active drag.
- First-party proof surfaces still cannot teach an ImGui/egui-class source preview outcome.

## Fret conclusion

The repo no longer needs a new drag payload lane.
It needs a source preview lane.

The correct owner split is:

- `fret-ui-kit::imui`: read-only drag observation seam only
- `fret-ui-kit::recipes`: first public drag preview ghost helper
- app/product code: preview content and domain semantics

The correct first-slice scope is:

- same-window preview ghost,
- source-authored content,
- click-through overlay behavior,
- and no cross-window preview choreography in v1.

## Immediate decision consequence

This audit supports the following design direction:

1. Do not widen runtime drag contracts for preview-specific needs.
2. Do not move preview-policy knobs onto `DragSourceOptions`.
3. Prefer exposing drag pointer position through `DragSourceResponse` or an equivalent small seam.
4. Build the first public ghost helper one layer up in `fret-ui-kit::recipes`.
5. Treat cross-window preview, multi-item aggregate preview, and shell-specific preview behavior as
   future contract work, not as part of the first same-window ghost slice.
