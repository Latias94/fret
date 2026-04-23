# ImUi Collection Delete Action v1

Status: closed closeout reference
Last updated: 2026-04-22

Status note (2026-04-22): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md` and
`CLOSEOUT_AUDIT_2026-04-22.md`. References below to broader collection action semantics should be
read as lane-opening rationale rather than an invitation to reopen either the collection
keyboard-owner folder or the generic key-owner / shared-helper questions.

Related:

- `M0_BASELINE_AUDIT_2026-04-22.md`
- `M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`
- `CLOSEOUT_AUDIT_2026-04-22.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_delete_action_surface.rs`
- `repo-ref/imgui/imgui_demo.cpp`

This lane exists because the closed collection keyboard-owner follow-on already proved the current
asset-browser surface can own deeper collection policy locally, but it explicitly deferred richer
collection action semantics.

The narrow remaining question is now:

> land one app-owned collection delete-selected slice on the existing proof surface, keep the
> generic key-owner and shared-helper widening verdicts closed, and avoid turning one demo's
> collection action semantics into a new public `fret-ui-kit::imui` command facade.

## Why this is a new lane

This work should not be forced back into `imui-collection-keyboard-owner-v1`.

That folder is already closed on a bounded keyboard-owner verdict. Reopening it would blur:

- collection depth already shipped
  - focusable collection-scope keyboard owner, active-tile movement, shift-range extension, and
    clear-on-escape;
- collection depth still open
  - delete-selected action semantics, broader select-all / rename / context menu action posture,
    and any future shared helper pressure.

This work also should not reopen the generic key-owner lane or widen runtime contracts.
The question here is narrower and first-party:

- does the current collection-first proof need one app-owned delete-selected slice,
- while still leaving generic key-owner APIs, runtime widening, and shared collection command
  helpers closed?

## Assumptions-first baseline

### 1) The closed collection keyboard-owner lane already deferred collection action semantics.

- Evidence:
  - `docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would duplicate or muddy the keyboard-owner closeout instead of owning the next
    narrow collection slice.

### 2) The proof-budget rule and runtime contract posture remain unchanged for this lane.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would drift from proof-surface policy into runtime or public-surface widening.

### 3) The current proof surface is still the narrowest correct owner.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would promote delete semantics into shared helper code before repeated first-party
    proof exists.

### 4) Dear ImGui keeps delete requests at the collection proof surface rather than using them as a reason to widen unrelated runtime or shared-helper contracts.

- Evidence:
  - `repo-ref/imgui/imgui_demo.cpp`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane could overfit one demo instead of following the upstream parity posture.

### 5) The lane should land and close narrowly.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the folder would expand into select-all, rename, context menu commands, or helper growth
    instead of one reviewable delete slice.

## Goals

1. Land one app-owned delete-selected slice on the current collection-first proof.
2. Keep the implementation explicit in `apps/fret-examples/src/imui_editor_proof_demo.rs`.
3. Freeze that the generic key-owner and shared-helper widening verdicts remain intact.
4. Leave one repro, one gate package, and one evidence set for the slice.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with a collection action or command helper.
- Reopening the generic key-owner question.
- Solving lasso / freeform drag-rectangle policy.
- Solving select-all, rename, or context-menu command breadth.
- Reopening the broader collection/pane proof closeout.

## Initial target surface

The current collection proof already has the ingredients this slice needs:

- stable item ids,
- visible-order reversal,
- `ImUiMultiSelectState<Arc<str>>`,
- app-owned background focus and box-select scope,
- and the new app-owned collection keyboard state from the closed follow-on.

The first landable target is therefore narrow:

1. make `Delete` / `Backspace` remove the current selected set in visible collection order,
2. add one explicit button-owned affordance for the same action,
3. keep asset storage, next selection, and next keyboard active tile app-owned,
4. keep the action explicit in the proof demo instead of inventing a generic command bus,
5. and leave select-all / rename / menu-owned action semantics for different follow-ons.

## Default owner split

### `apps/fret-examples`

Owns:

- the mutable collection asset model,
- the delete-selection helper and reflow policy,
- the explicit button affordance,
- and the source-policy teaching surface for this lane.

### `fret-ui-kit::imui`

Owns:

- the existing multi-select, selectable, drag/drop, and child-region seams this proof builds on,
- but not a new public collection action helper in this lane.

### Not owned here

- `docs/workstreams/imui-collection-keyboard-owner-v1/`
  - remains the closed keyboard-owner verdict.
- `docs/workstreams/imui-key-owner-surface-v1/`
  - remains the closed generic key-owner verdict.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- select-all / rename / context menu commands
  - remain separate follow-ons if stronger first-party proof appears later.

## First landable target

Do not start by designing a shared collection command facade or helper.

The first correct target is:

- one proof-local asset-storage model plus delete helper inside
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- explicit `Delete` / `Backspace` handling plus one visible action button,
- source-policy and unit-test gates that keep the slice visible,
- and a closeout that freezes generic key-owner / shared-helper widening as still unjustified.
