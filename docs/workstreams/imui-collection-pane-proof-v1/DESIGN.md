# ImUi Collection + Pane Proof v1

Status: active execution lane
Last updated: 2026-04-21

Related:

- `M0_BASELINE_AUDIT_2026-04-21.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`

This lane exists because the maintenance umbrella already narrowed the remaining P0/P1 backlog and
explicitly said that implementation-heavy collection/pane proof work should move into a smaller
follow-on instead of widening the umbrella again.

The smallest credible follow-on is now:

> prove editor-grade collection and pane composition breadth with real first-party surfaces,
> decide whether any helper widening is actually justified, and keep that work out of
> `crates/fret-ui`, the closed shell-helper lane, and the active multi-window runner lane.

## Why this is a new lane

This should not be forced back into `imui-editor-grade-product-closure-v1` because the remaining
question is now implementation-heavy and tightly scoped around proof breadth.

It also should not be mixed with the separate key-owner problem.
The current evidence does not justify bundling together:

- collection proof breadth,
- child-region / pane proof depth,
- key ownership,
- richer menubar/tab policy,
- and shell-helper promotion

into one broad "remaining imgui parity" folder again.

This lane is narrower than the umbrella:

- the umbrella keeps phase ordering and cross-phase status,
- this lane owns only collection/pane proof breadth and the first bounded implementation slices
  that may follow from it.

## Assumptions-first baseline

### 1) The missing gap is proof breadth, not runtime absence.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
  - `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would under-scope a real mechanism gap and leave the wrong owner in place.

### 2) `crates/fret-ui` must remain unchanged unless stronger ADR-backed evidence appears.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would drift into runtime widening instead of ecosystem-level proof closure.

### 3) Shell-helper promotion is already closed for now.

- Evidence:
  - `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would start re-arguing helper extraction instead of proving the current starter set.

### 4) Key ownership remains a separate follow-on unless new evidence proves otherwise.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - this folder would turn back into a generic immediate-convenience backlog.

### 5) Multi-window parity remains in the active docking lane.

- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
- Confidence:
  - Confident
- Consequence if wrong:
  - collection/pane proof work would get mixed with runner/backend follow behavior.

## Goals

1. Freeze the owner split for editor-grade collection/pane proof breadth.
2. Build one collection-first proof surface and one pane-first proof surface that are reviewable as
   real first-party evidence instead of isolated helper demos.
3. Decide whether any helper widening is actually required after those proofs exist.
4. Leave one repro set, one gate package, and one evidence set for the lane.

## Non-goals

- Widening `crates/fret-ui`.
- Reopening the no-new-helper-yet shell verdict.
- Reopening the active multi-window docking parity lane.
- Solving key-owner/global shortcut ownership in this lane.
- Turning richer menubar/tab policy into an implicit side quest of collection/pane proof work.

## Initial target surface

This lane does not start from zero.
The immediate stack already has reusable collection/pane ingredients:

- keyed identity and collection helpers,
- `ImUiMultiSelectState<K>` plus model-backed collection selection,
- `child_region[_with_options]`,
- floating areas/windows,
- tables and virtual lists,
- editor adapters in `fret-ui-editor::imui`,
- workspace shell composition surfaces in `fret-workspace`.

The missing gap is the narrower proof pair around:

1. **Collection-first proof**
   - an asset-grid / file-browser style surface,
   - real multi-select breadth beyond row lists,
   - a clear decision on marquee / box-select bridging,
   - and a first-party proof that combines collection selection with drag/drop and editor-owned
     details.

2. **Pane-first proof**
   - a `BeginChild()`-scale composition story over the existing `child_region` seam,
   - toolbar / status / tabs / inspector style nested panes,
   - and a shell-mounted proof that exercises the pane composition without reopening helper
     promotion.

The lane starts from the current first-open proof pair:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`

Those are the baseline surfaces to refine or supersede with narrower proof demos.

## Default owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- collection/pane helper seams such as `multi_select` and `child_region`,
- any facade-only helper widening justified by the proof pair,
- and the bounded policy needed to keep those helpers coherent.

### `ecosystem/fret-imui`

Owns:

- focused interaction tests for the collection/pane proof closures,
- drag/drop, selection, and floating interaction floors those proofs depend on,
- and proof that the facade semantics remain stable across real interactions.

### `ecosystem/fret-ui-editor`

Owns:

- editor-grade composites mounted inside the proof surfaces,
- inspector/property-grid style content used to make the proofs feel like real editor work,
- and any editor-local composition seams that do not belong in generic `imui`.

### `apps/fret-examples`

Owns:

- the first-party proof demos and source-policy teaching surfaces,
- the current baseline pair (`imui_editor_proof_demo` and `workspace_shell_demo`),
- and any narrower proof demo promoted by this lane.

### `ecosystem/fret-workspace`

Owns:

- the shell mounting surfaces and starter-set composition used by pane proofs,
- but not a new promoted workbench helper in this lane.

### Not owned here

- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- shell-helper promotion
  - still closed unless repeated stronger evidence appears later.
- key-owner/global shortcut ownership
  - still separate unless another narrow follow-on proves it should move.
- runner/backend multi-window hand-feel
  - still owned by `docs/workstreams/docking-multiwindow-imgui-parity/`.

## Execution rules

1. Use the umbrella lane for cross-phase status and follow-on policy.
2. Use this lane only for collection/pane proof breadth and the first bounded implementation slices
   that fall out of those proofs.
3. If pressure shifts to key ownership, start a separate narrow follow-on.
4. If pressure shifts to promoted shell helpers, return to the closed shell verdict and require
   stronger repeated evidence before opening a different lane.
5. Every slice in this lane must name:
   - one collection or pane proof surface,
   - one focused gate package,
   - and one evidence set.
