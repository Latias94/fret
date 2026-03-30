# imui drag preview ghost v1 - milestones

Status: active progress record

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-drag-preview-ghost-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-drag-preview-ghost-v1/TODO.md`

Upstream parity audit:

- `docs/workstreams/imui-drag-preview-ghost-v1/UPSTREAM_PARITY_AUDIT_2026-03-30.md`

Predecessor closeouts:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`

## Phase A - Workstream setup and owner freeze

Status: Completed

Goal:

- open the ghost lane explicitly,
- freeze the owner split before code lands,
- and keep source preview from drifting into `imui` policy sprawl.

Deliverables:

- one new workstream directory with design/TODO/milestones,
- one explicit owner split for `imui` seam vs recipe ghost helper,
- one README/workstream-map update that points readers to this active lane.

Exit gates:

- the repo can explain why source preview is a new lane after sortable closeout,
- and the first proof/gate package is named before implementation starts.

## Phase B - Stable contract freeze

Status: Completed

Goal:

- freeze the smallest drag preview contract worth shipping.

Deliverables:

- one upstream parity read against Dear ImGui and egui,
- one decision on public owner (`recipes`) vs support seam (`imui`),
- one preferred support-seam direction for drag-position visibility,
- one explicit defer list for cross-window and shell-specific preview choreography.

Current direction:

- the first public ghost helper should live in `ecosystem/fret-ui-kit::recipes`,
- `fret-ui-kit::imui` may add only a small read-only drag-position seam,
- and the first slice is intentionally same-window only.

Exit gates:

- the workstream can explain the contract without widening runtime seams,
- source preview remains source-authored,
- and the deferred list is short and explicit.

Completion notes:

- the public recipe names are frozen as
  `DragPreviewGhostOptions`, `drag_preview_ghost(...)`, and
  `drag_preview_ghost_with_options(...)`,
- `DragSourceResponse::position()` is the only new support seam in `imui`,
- and `start_position` was intentionally not added for v1.

## Phase C - Proof-first implementation

Status: Completed

Goal:

- land the ghost where it actually improves immediate authoring.

Deliverables:

- minimal support seam in `fret-ui-kit::imui`,
- one reusable ghost helper in `ecosystem/fret-ui-kit::recipes`,
- migrated first-party proof/demo surfaces for at least one asset drag and one row drag.

Exit gates:

- the proof surfaces are materially clearer than app-local overlay glue,
- the ghost helper composes with the current typed drag seam,
- and `DragSourceOptions` does not turn into a preview-policy bag.

Completion notes:

- the asset drag proof and reorderable outliner proof now both render source-authored ghosts,
- preview content stays at the source call site via `IntoUiElement` authoring,
- and no compatibility alias or preview knob was added to `drag_source(...)`.

## Phase D - Gates and explicit closeout/defer

Status: In progress

Goal:

- leave a durable proof/gate package and document what remains after the first slice.

Deliverables:

- focused unit + smoke coverage,
- one real interaction gate,
- explicit defer notes for wider preview choreography,
- and a closeout audit once the first slice is shipped or intentionally deferred.

Exit gates:

- the new helper is reviewable through docs + proof + gate,
- the lower typed drag/drop seam remains clean,
- and any surviving gaps are clearly identified as future contract lanes instead of hidden backlog.

Current status notes:

- focused unit coverage, compile-surface smoke coverage, and a real `fret-imui` interaction gate
  are landed,
- `window_overlays::render` now explicitly allows non-interactive hover/tooltip overlays to remain
  visible during pointer capture so drag ghosts can render without widening runtime contracts,
- the remaining open artifact for this phase is a dedicated closeout audit document.
