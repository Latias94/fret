# imui editor-grade surface closure v1 - milestones

Tracking doc: `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-editor-grade-surface-closure-v1/TODO.md`

Gap audit: `docs/workstreams/imui-editor-grade-surface-closure-v1/EDITOR_GRADE_GAP_AUDIT_2026-03-29.md`

Predecessor:

- `docs/workstreams/imui-stack-fearless-refactor-v1/`

This file is forward-looking only.
The predecessor stack-reset workstream is already closed; this follow-on exists to close the
remaining editor-grade gaps without reopening the completed surface-cleanup effort.

## Phase A - Scope freeze and gap classification

Status: Completed

Goal:

- freeze the follow-on scope,
- classify each gap by owning crate,
- and avoid mixing generic immediate helpers with docking/workspace shell policy.

Deliverables:

- one explicit gap classification matrix,
- one owner split for composite adapters vs generic helpers vs docking/workspace-owned surfaces,
- one proof-surface list that this workstream must simplify.

Exit gates:

- the team can explain why each target belongs in its chosen crate,
- non-goals are explicit,
- and the workstream is clearly about closure rather than another broad redesign.

## Phase B - Editor composite adapter closure

Status: Completed

Goal:

- make the core editor composites usable from immediate-mode authoring without re-implementing them.

Deliverables:

- thin adapters for `PropertyGroup`, `PropertyGrid`, `PropertyGridVirtualized`, and `InspectorPanel`,
- explicit decision to keep `GradientEditor` declarative-only for now,
- one source-policy test that keeps composite adapters thin,
- migrated first-party proof/demo call sites for the promoted composites.

Exit gates:

- the promoted composites can be authored through `fret-ui-editor::imui`,
- the adapter file remains a one-hop forwarding layer,
- and no declarative composite gains a second implementation path.

## Phase C - Generic editor-shell helper closure

Status: Planned

Goal:

- close the generic immediate helper gaps that affect editor hand-feel across crates.

Deliverables:

- a first-class tooltip helper,
- a first-class collapsing/tree immediate family,
- explicit authoring guidance for stable IDs in tree/outliner scenarios,
- owner decisions for any shell-like helper that should be routed to docking/workspace instead.

Exit gates:

- tooltip and tree/outliner authoring no longer require ad hoc per-call-site glue,
- `fret-imui` itself remains minimal,
- and the generic helper set still fits `fret-ui-kit::imui` rather than turning into shell policy.

## Phase D - Drag/drop decision and closeout

Status: Planned

Goal:

- either land a clean immediate drag/drop authoring seam or document an explicit defer boundary.

Deliverables:

- one runtime-boundary audit result,
- either a typed immediate drag/drop helper family or a defer note,
- proof on at least one editor-grade scenario,
- final closeout notes and updated immediate-mode map/docs.

Exit gates:

- drag/drop is either cleanly shipped or cleanly deferred,
- proof/demo surfaces show the new editor-grade closure,
- and the remaining `imui` gap list is short, explicit, and correctly owned.
