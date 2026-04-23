# ImUi Collection Rename v1

Status: closed closeout reference
Last updated: 2026-04-23

Status note (2026-04-23): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_APP_OWNED_RENAME_SLICE_2026-04-23.md` and `CLOSEOUT_AUDIT_2026-04-23.md`.
References below to broader collection/product depth should be read as lane-opening rationale
rather than an invitation to reopen either the collection select-all folder or generic
key-owner/helper ownership questions.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M1_APP_OWNED_RENAME_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`
- `repo-ref/imgui/imgui.h`

This lane exists because the closed collection select-all follow-on already proved the current
asset-browser proof can own deeper collection action breadth locally, but it explicitly deferred
rename breadth.

The narrow remaining question is now:

> land one app-owned collection rename slice on the existing proof surface, keep generic
> key-owner/helper widening closed, and avoid turning one demo's `F2` plus popup/input affordance
> into a new public `fret-ui-kit::imui` collection helper or runtime contract.

## Why this is a new lane

This work should not be forced back into `imui-collection-select-all-v1`.

That folder is already closed on a bounded select-all verdict. Reopening it would blur:

- collection depth already shipped
  - background marquee / box-select, collection keyboard owner, delete-selected, context-menu,
    zoom/layout, select-all;
- collection depth still open
  - rename breadth, any second proof surface strong enough to justify shared helper growth, and
    any broader collection command/helper package.

This work also should not widen `crates/fret-ui` or generic `fret-ui-kit::imui` collection policy.

## Assumptions-first baseline

### 1) The closed collection select-all lane already deferred rename breadth.

- Evidence:
  - `docs/workstreams/imui-collection-select-all-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would duplicate or muddy the select-all closeout instead of owning the next narrow
    collection slice.

### 2) The current proof surface already has the right ingredients for a narrow app-owned collection rename slice:

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would invent extra helper surface instead of exercising the already-shipped
    collection-scope key ownership plus popup/input seams from one real proof surface.

### 3) The current proof already has popup and text-input seams, so this lane is not a justification to widen shared helper ownership.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would accidentally reopen generic popup/input/runtime questions for a product-owned
    asset browser depth issue.

### 4) Dear ImGui keeps rename breadth close to the current proof surface instead of turning it into a generic runtime contract.

- Evidence:
  - `repo-ref/imgui/imgui.h`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane could overfit local demo structure rather than following the upstream parity posture.

### 5) The lane should land and close narrowly.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the folder would expand into a generic inline-edit helper or second proof surface before one
    reviewable rename slice ships.

## Goals

1. Land one app-owned collection rename slice on the current collection-first proof.
2. Route F2 through the existing collection-scope keyboard owner.
3. Reuse the current popup plus text-input seams instead of widening helper/runtime ownership.
4. Leave one repro, one gate package, and one evidence set for the slice.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with a collection rename helper.
- Designing a generic inline-edit or collection command package.
- Reopening the popup/menu lane for broader command breadth.
- Adding a second proof surface.

## Initial target surface

The current collection proof already has the ingredients this slice needs:

- stable item ids,
- a collection-scoped keyboard owner,
- an existing context menu,
- existing text-input/popup seams,
- and a closed proof-budget rule that still blocks shared helper growth.

The first landable target is therefore narrow:

1. route F2 through the existing collection-scope keyboard owner,
2. open one app-owned rename modal from the current active asset or context-menu selection,
3. commit one label-only rename while preserving stable ids and visible order,
4. keep the popup/input surface product-owned in this lane,
5. and leave second-proof-surface / broader helper pressure for different follow-ons.

## Default owner split

### `apps/fret-examples`

Owns:

- the rename shortcut matcher,
- rename session targeting,
- proof-local rename commit policy,
- and the source-policy teaching surface for this lane.

### `fret-ui-kit::imui`

Owns:

- the existing popup modal and input seams this proof already uses,
- but not a new public collection rename helper in this lane.

### Not owned here

- `docs/workstreams/imui-collection-select-all-v1/`
  - remains the closed select-all verdict.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- second proof surface / broader collection command package
  - remain separate follow-ons if stronger first-party proof appears later.

## First landable target

Do not start by designing a shared collection rename helper or generic inline-edit surface.

The first correct target is:

- one proof-local `F2` matcher plus one proof-local rename session/commit helper in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- a modal opened from the current active asset or context-menu selection,
- source-policy and unit-test gates that keep the slice visible,
- and a closeout that freezes generic key-owner/helper widening as still unjustified.
