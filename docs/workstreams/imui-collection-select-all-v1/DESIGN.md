# ImUi Collection Select-All v1

Status: closed closeout reference
Last updated: 2026-04-23

Status note (2026-04-23): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_APP_OWNED_SELECT_ALL_SLICE_2026-04-23.md` and `CLOSEOUT_AUDIT_2026-04-23.md`.
References below to broader collection/product depth should be read as lane-opening rationale
rather than an invitation to reopen either the collection zoom folder or generic key-owner/helper
ownership questions.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M1_APP_OWNED_SELECT_ALL_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_select_all_surface.rs`
- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui_widgets.cpp`

This lane exists because the closed collection zoom follow-on already proved the current
asset-browser proof can own deeper collection layout policy locally, but it explicitly deferred
select-all breadth.

The narrow remaining question is now:

> land one app-owned collection select-all slice on the existing proof surface, keep generic
> key-owner/helper widening closed, and avoid turning one demo's `Primary+A` affordance into a new
> public `fret-ui-kit::imui` collection helper or runtime contract.

## Why this is a new lane

This work should not be forced back into `imui-collection-zoom-v1`.

That folder is already closed on a bounded zoom/layout verdict. Reopening it would blur:

- collection depth already shipped
  - background marquee / box-select, collection keyboard owner, delete-selected, context-menu,
    zoom/layout;
- collection depth still open
  - select-all breadth, rename breadth, and any second proof surface strong enough to justify
    shared helper growth.

This work also should not widen `crates/fret-ui` or generic `fret-ui-kit::imui` collection policy.

## Assumptions-first baseline

### 1) The closed collection zoom lane already deferred collection select-all breadth.

- Evidence:
  - `docs/workstreams/imui-collection-zoom-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would duplicate or muddy the zoom closeout instead of owning the next narrow
    collection slice.

### 2) The current proof surface already has the right ingredients for a narrow app-owned collection select-all slice:

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would invent extra helper surface instead of exercising the already-shipped
    collection-scope key ownership and visible-order math from one real proof surface.

### 3) The collection-scope key-owner and visible-order helpers already exist locally, so this lane is not a justification to widen shared helper ownership.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would accidentally reopen generic key-owner/runtime questions for a product-owned
    asset browser depth issue.

### 4) Dear ImGui keeps Ctrl+A selection breadth in the multi-select proof surface instead of turning it into a generic runtime contract.

- Evidence:
  - `repo-ref/imgui/imgui_demo.cpp`
  - `repo-ref/imgui/imgui_widgets.cpp`
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
  - the folder would expand into rename or generic helper growth instead of one reviewable
    select-all slice.

## Goals

1. Land one app-owned collection select-all slice on the current collection-first proof.
2. Route `Primary+A` through the existing collection-scope owner instead of widening generic
   key-owner APIs.
3. Reuse visible-order collection math so select-all follows the current browser order.
4. Leave one repro, one gate package, and one evidence set for the slice.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with a collection select-all helper.
- Reopening the generic key-owner question.
- Reopening the popup/menu lane for broader command breadth.
- Solving rename, inline editing, or broader command palette integration.
- Adding a second proof surface.

## Initial target surface

The current collection proof already has the ingredients this slice needs:

- stable item ids,
- a collection-scoped keyboard owner,
- visible-order collection math,
- existing selection/active-tile models,
- and a closed proof-budget rule that still blocks shared helper growth.

The first landable target is therefore narrow:

1. route Primary+A through one collection-scope select-all policy,
2. select all visible assets in current visible order,
3. keep the current active tile when possible instead of widening generic key-owner ownership,
4. keep the popup/menu surface unchanged in this lane,
5. and leave rename / broader command breadth / second-proof-surface pressure for different
   follow-ons.

## Default owner split

### `apps/fret-examples`

Owns:

- the select-all shortcut matcher,
- the visible-order select-all selection helper,
- active-tile preservation policy,
- and the source-policy teaching surface for this lane.

### `fret-ui-kit::imui`

Owns:

- the existing selectables and collection-scope key hooks this proof already uses,
- but not a new public collection select-all helper in this lane.

### Not owned here

- `docs/workstreams/imui-collection-zoom-v1/`
  - remains the closed zoom/layout verdict.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- rename / second proof surface
  - remain separate follow-ons if stronger first-party proof appears later.

## First landable target

Do not start by designing a shared collection select-all helper or broader command surface.

The first correct target is:

- one proof-local shortcut matcher plus one proof-local select-all helper in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- `Primary+A` handled by the existing collection-scope keyboard owner,
- source-policy and unit-test gates that keep the slice visible,
- and a closeout that freezes generic key-owner/helper widening as still unjustified.
