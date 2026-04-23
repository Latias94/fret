# ImUi Collection Second Proof Surface v1

Status: historical execution reference (closed lane)
Last updated: 2026-04-23

Status note (2026-04-23): this lane starts immediately after the bounded command-package lane
closed. Its job is to freeze and then land a materially different second collection proof surface
on an existing shell-mounted demo before any shared collection helper widening can reopen.
The M2 slice landed that first shell-mounted collection surface in `editor_notes_demo.rs`, and
`CLOSEOUT_AUDIT_2026-04-23.md` now closes this lane on a no-helper-widening verdict.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`

This lane exists because:

The closed command-package lane already proved that one collection-first proof surface can carry a
coherent app-owned duplicate-plus-rename package locally.

The next narrow question is now:

> freeze and then land a second real collection proof surface on an existing shell-mounted demo,
> keep helper/runtime widening closed, and decide future generic collection growth from two real
> proof surfaces instead of from one increasingly heavy demo.

## Why this is a new lane

This work should not be forced back into
`imui-collection-command-package-v1`.

That folder is now closed on a bounded command-package verdict. Reopening it would blur:

- the landed duplicate-selected plus explicit rename-trigger package,
- the separate proof-budget requirement for a second real surface,
- and the difference between product proof breadth versus shared helper growth.

This lane also should not create a new dedicated asset-grid/file-browser demo.
The current question is whether an existing shell-mounted proof can carry materially different
collection pressure, not whether the repo can invent another synthetic showcase.

## Assumptions-first baseline

### 1) The command-package lane is now closed, so the next default non-multi-window priority is a second proof surface.

- Evidence:
  - `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would reopen a closed command-package folder instead of owning the next product-depth
    question.

### 2) `editor_notes_demo.rs` is the smallest materially different shell-mounted second proof candidate currently in tree.

- Evidence:
  - `apps/fret-examples/src/editor_notes_demo.rs`
  - `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would pick a heavier shell before exhausting the smallest materially different
    existing proof.

### 3) `workspace_shell_demo.rs` remains supporting evidence, but it should not be the only second proof candidate.

- Evidence:
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
  - `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would confuse “broader shell proof” with “second materially different collection
    proof.”

### 4) The frozen proof-budget rule still blocks shared collection helper widening until a second real proof surface exists.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would widen `fret-ui-kit::imui` too early instead of strengthening first-party proof.

## Goals

1. Freeze the second proof-surface owner question as a shell-mounted collection follow-on rather
   than a helper-growth question.
2. Prefer existing demos, with `editor_notes_demo.rs` as the primary candidate and
   `workspace_shell_demo.rs` as supporting proof.
3. Keep the work app-owned inside `apps/fret-examples`.
4. Leave one repro, one gate floor, and one evidence set for the lane before code implementation
   broadens.

## Non-goals

- Reopening `imui-collection-command-package-v1`.
- Creating a dedicated asset-grid/file-browser proof demo.
- Widening `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.
- Treating runner/backend multi-window gaps as part of this lane.
- Promoting new generic workspace-shell helpers from one second proof candidate.

## Landed target surface

The current target is therefore proof-surface evidence rather than helper growth:

1. keep `apps/fret-examples/src/editor_notes_demo.rs` as the primary shell-mounted second
   collection proof surface,
2. keep the landed `Scene collection` left-rail surface app-owned inside that demo,
3. keep `apps/fret-examples/src/workspace_shell_demo.rs` as supporting shell-mounted collection
   evidence,
4. freeze that no dedicated asset-grid/file-browser demo should be introduced yet,
5. and require a separate post-proof decision before any shared collection helper widening can
   reopen.

## Default owner split

### `apps/fret-examples`

Owns:

- the second proof-surface candidate selection,
- shell-mounted collection composition inside existing demos,
- and the source-policy teaching surface for this lane.

### Not owned here

- `fret-imui`
  - no new generic collection facade.
- `fret-ui-kit::imui`
  - do not introduce shared collection helpers from this lane.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- runner/backend multi-window parity
  - remains owned by the docking parity lane.
