# ImUi Collection Box Select v1

Status: closed closeout reference
Last updated: 2026-04-22

Status note (2026-04-22): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md` and `CLOSEOUT_AUDIT_2026-04-22.md`.
References below to broader collection-depth work should be read as lane-opening rationale rather
than an invitation to widen this folder back into a generic collection backlog.

Related:

- `M0_BASELINE_AUDIT_2026-04-22.md`
- `M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
- `CLOSEOUT_AUDIT_2026-04-22.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_box_select_surface.rs`
- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui.h`

This lane exists because the closed collection/pane proof lane already proved that the current
collection-first asset browser is a real first-party proof, but it explicitly deferred marquee /
box-select. The remaining collection gap is therefore narrower than "prove asset-browser breadth at
all":

> land one app-owned background marquee / box-select slice on the existing proof surface, keep the
> current proof-budget rule intact, and avoid turning one demo's collection policy into a new
> generic `fret-ui-kit::imui` helper prematurely.

## Why this is a new lane

This work should not be forced back into `imui-collection-pane-proof-v1`.

That folder is already closed on a no-helper-widening verdict for the broader collection/pane proof
pair. Reopening it would blur two different questions:

- proof breadth
  - already closed by the asset-browser and shell-mounted pane proofs;
- collection depth
  - now narrowed to a specific background box-select story on the collection proof surface.

This lane also should not absorb:

- lasso / freeform drag-rectangle policy,
- collection keyboard-owner / richer shortcut ownership,
- runner/backend multi-window parity,
- or new public helper growth in `fret-ui-kit::imui`.

## Assumptions-first baseline

### 1) The missing gap is narrower collection depth, not runtime absence.

- Evidence:
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would under-scope a real mechanism gap and leave the wrong owner in place.

### 2) The frozen two-surface proof budget blocks a new public `fret-ui-kit::imui` helper here.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
  - `apps/fret-cookbook/examples/imui_action_basics.rs`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would widen public surface from one proof surface and erode the frozen budget rule.

### 3) The narrowest correct owner is the current app proof surface.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would promote policy into shared helper code before repeated first-party pressure
    exists.

### 4) `crates/fret-ui` must stay unchanged unless ADR-backed evidence proves a mechanism gap.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would drift from collection policy into runtime widening.

### 5) This lane should close on background-only box-select, not reopen broader collection depth.

- Evidence:
  - `repo-ref/imgui/imgui_demo.cpp`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the folder would turn back into a generic lasso / keyboard-owner / helper-growth backlog.

## Goals

1. Land one app-owned background marquee / box-select proof slice on the existing collection-first
   asset browser.
2. Keep the implementation explicit in `apps/fret-examples/src/imui_editor_proof_demo.rs` instead
   of promoting a new public helper.
3. Leave one repro, one gate package, and one evidence set that prove the slice is intentional and
   reviewable.

## Non-goals

- Widening `fret-ui-kit::imui` with a new public collection box-select helper.
- Widening `crates/fret-ui`.
- Solving lasso / freeform drag-rectangle policy.
- Solving collection keyboard-owner or richer item-local shortcut ownership.
- Reopening the broader collection/pane proof lane.
- Reopening runner/backend multi-window parity.

## Initial target surface

The current collection proof already has the right first-party ingredients:

- stable item ids,
- `ImUiMultiSelectState<Arc<str>>`,
- selected-set drag/drop,
- explicit visible-order reversal,
- and a reviewable asset-grid/file-browser style proof surface.

The first landable target is therefore narrow:

1. mount a background pointer region around the current collection browser content,
2. treat box-select as background-only policy so clicking directly on asset tiles still uses the
   existing selectable row semantics,
3. keep selection updates visible-order normalized,
4. draw a marquee overlay while dragging,
5. and keep every piece app-owned in the proof demo.

## Default owner split

### `apps/fret-examples`

Owns:

- the background pointer region,
- the drag-session state and marquee overlay,
- the box-select selection policy over the asset grid,
- and the source-policy teaching surface for this lane.

### `fret-ui-kit::imui`

Owns:

- the existing multi-select, drag/drop, and child-region seams this proof is built on,
- but not a new public box-select helper in this lane.

### `fret-imui`

Owns:

- the shared interaction floors already shipped for immediate drag/drop and selection,
- but not a new generic collection box-select surface in this lane.

### Not owned here

- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- lasso / keyboard-owner depth
  - still separate follow-ons if stronger first-party proof appears later.
- additional proof surfaces
  - still required before public helper widening can be reconsidered.

## First landable target

Do not begin by designing a shared helper surface.

The first correct target is:

- a background-only marquee / box-select slice inside
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- explicit app-owned state and pointer-region policy,
- source-policy and unit-test gates that keep the slice visible,
- and a closeout that freezes helper widening as still unjustified on current proof budget.
