# imui editor-grade gap audit — 2026-03-29

Status: focused audit note
Last updated: 2026-03-29

Related:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`

## Why this note exists

The completed stack-reset workstream answered the structural question:

> Is the `imui` stack coherent and contract-clean?

The remaining product question is different:

> Relative to Dear ImGui and egui, what editor-grade immediate surfaces are still missing or
> underpowered now that the stack itself is clean?

This note records the current answer so the next workstream starts from a concrete gap list instead
of intuition.

## Current conclusion

The current gap is **not** "we still need many more primitive widgets".

The current gap is:

- editor composites,
- tree/collapsing authoring,
- tooltip packaging,
- and drag/drop authoring seams.

In other words:

- the primitive floor is already decent,
- but the editor-grade skeleton is still incomplete.

## Bucket 1: already strong enough for the next phase

Current strengths:

- minimal frontend + shared contract
- floating windows / in-window floating areas
- popup/menu behavior
- disabled scopes
- response-query semantics
- keyed identity helpers
- basic immediate form controls
- promoted editor scalar/vector controls

Evidence:

- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Practical implication:

- the next closure batch should not spend its main budget on more button/text-input-level widgets.

## Bucket 2: highest-priority missing surfaces

### A. Editor composites

Current issue:

- editor proof/demo authoring still falls back to declarative composite composition for inspector
  skeletons.

Highest-value targets:

- `PropertyGroup`
- `PropertyGrid`
- `PropertyGridVirtualized`
- `InspectorPanel`

Owner:

- `ecosystem/fret-ui-editor::imui`

Reason:

- these are editor-specific declarative composites that already exist in the editor crate,
- so the immediate side should be thin adapter coverage, not a new generic facade invention.

Evidence:

- `ecosystem/fret-ui-editor/src/composites/mod.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

### B. Tree / collapsing authoring

Current issue:

- there is no first-class generic immediate tree/collapsing family even though this is a common
  editor and egui/imgui workflow.

Owner:

- `ecosystem/fret-ui-kit::imui`

Reason:

- tree/collapsing is generic immediate authoring vocabulary, not editor-only declarative composite
  ownership.

Evidence:

- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `repo-ref/egui/crates/egui/src/containers/collapsing_header.rs`

### C. Tooltip helper

Current issue:

- the hover/query substrate already knows enough about tooltip-style delay behavior,
- but there is no first-class helper that turns that substrate into a clean immediate authoring
  surface.

Owner:

- `ecosystem/fret-ui-kit::imui`

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`

### D. Drag/drop payload authoring

Current issue:

- editor-grade drag/drop workflows still have no explicit immediate payload authoring contract.

Current status:

- the runtime-boundary audit is now recorded in
  `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`,
- and the first clean typed source/target seam now ships from `fret-ui-kit::imui`.

Owner:

- `ecosystem/fret-ui-kit::imui`

Reason:

- the authoring surface is generic immediate vocabulary,
- the shipped helper reuses existing runtime drag-session routing,
- and richer collision/sortable/workspace policy remains correctly outside this crate.

Evidence:

- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Bucket 3: defer unless fresh evidence appears

These surfaces may still matter, but they should not lead the next workstream:

- more primitive leaf widgets for their own sake,
- style-stack clones,
- broad `WindowFlags` mirroring,
- last-item implicit context tricks,
- docking/workspace tab bars disguised as generic immediate helpers.

Reason:

- they either reopen already-closed structural decisions,
- or belong to other owner crates,
- or improve API familiarity more than they improve real editor authoring throughput.

## Practical priority order

1. composite adapters (`fret-ui-editor::imui`)
2. tooltip helper (`fret-ui-kit::imui`)
3. tree/collapsing helper family (`fret-ui-kit::imui`)
4. drag/drop boundary audit and decision
5. anything else only after proof/demo surfaces confirm a remaining gap

## Maintainer rule

Before adding another `imui` helper, ask:

1. does this close a real editor workflow gap, or only increase primitive count?
2. does it belong in `fret-ui-kit::imui`, `fret-ui-editor::imui`, or another crate entirely?
3. can it stay thin over an existing declarative owner, or is the real bug lower in the stack?

If the answer to 1 is weak, or the answer to 2 is unclear, do not land the helper yet.
