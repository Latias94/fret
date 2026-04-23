# ImUi Collection Command Package v1

Status: historical execution reference (closed lane)
Last updated: 2026-04-23

Status note (2026-04-23): this document remains the lane-opening rationale for the bounded
duplicate-selected plus explicit rename-trigger package. The shipped verdict now lives in
`CLOSEOUT_AUDIT_2026-04-23.md`, and the default next non-multi-window follow-on now lives in
`docs/workstreams/imui-collection-second-proof-surface-v1/`.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md`
- `M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_command_package_surface.rs`

This lane exists because:

The closed collection modularization lane already proved the host file can stay slim without widening any public surface.

The narrow remaining question is now:

> land one broader app-owned collection command package on the existing proof surface, keep helper
> and runtime widening closed, and decide command-package maturity from first-party editor flow
> instead of from framework taste.

## Why this is a new lane

This work should not be forced back into
`imui-editor-proof-collection-modularization-v1`.

That folder is already closed on a structural maintenance verdict. Reopening it would blur:

- the landed demo-local owner split,
- broader command-package depth on top of the current selection owner,
- and the still-deferred second proof-surface requirement before any shared helper widening can
  reopen.

This lane also should not widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.
The current gap is product-owned command breadth, not a missing generic mechanism.

## Assumptions-first baseline

### 1) The closed collection modularization lane already reset the next priority to command-package breadth.

- Evidence:
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would duplicate a closed structural folder instead of owning the next product-depth
    step.

### 2) The current proof surface already has enough local substrate for broader command breadth.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
  - `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would invent framework helper growth before exhausting what one real proof surface
    can already own locally.

### 3) A proof-local command status model is sufficient for this lane.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would drift into clipboard, platform reveal, or generic command-bus questions too
    early.

### 4) The frozen proof-budget rule still blocks shared helper growth from one proof surface.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would misread one proof surface's product depth as framework API evidence.

## Goals

1. Land a broader app-owned collection command package on the current collection-first proof.
2. Keep the package local to `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`.
3. Reuse the current selection owner, keyboard owner, explicit buttons, and context menu.
4. Leave one repro, one gate floor, and one evidence set for the lane.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with collection command helpers.
- Reopening the structural modularization lane.
- Requiring a second proof surface inside this folder.
- Turning this lane into clipboard, platform reveal, or OS integration work.

## Initial target surface

The first landable target is therefore command-package breadth rather than helper growth:

1. land `Primary+D` duplicate-selected on the existing proof surface,
2. route the same duplicate command through the explicit button and context menu,
3. keep command status feedback app-owned on the collection module,
4. preserve stable ids plus visible-order selection semantics after duplication,
5. and leave any second proof-surface or shared-helper decision to later evidence.

## Default owner split

### `apps/fret-examples`

Owns:

- the proof-local duplicate-selected helper,
- command status feedback,
- keyboard/button/context-menu routing,
- and the source-policy teaching surface for this lane.

### Not owned here

- `fret-imui`
  - no new generic command facade.
- `fret-ui-kit::imui`
  - Do not introduce a shared `collection_commands(...)` or `duplicate_selected(...)` helper in `fret-ui-kit::imui`.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- second proof surface
  - remains a separate follow-on once this command breadth settles.

## First landable target

The first correct target is:

- one proof-local duplicate-selected helper plus one proof-local command status model in
  `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`,
- `Primary+D`, an explicit button, and a context-menu entry all routed through the same local
  command path,
- source-policy plus unit/surface gates that keep the slice visible,
- and the later closeout verdict that rejects a third command verb in this folder before moving
  default priority to the second proof surface.
