# ImUi Response Status Lifecycle v1

Status: active execution lane
Last updated: 2026-04-13

Related:

- `M0_BASELINE_AUDIT_2026-04-13.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/UIWRITER_RESPONSE_CONTRACT_CLOSEOUT_2026-03-29.md`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

This lane exists because the active editor-grade product-closure umbrella already narrowed the
remaining P0 backlog and explicitly said that implementation-heavy immediate-convenience work
should split into a smaller follow-on.

The smallest credible follow-on is now:

> freeze the missing item-status lifecycle vocabulary for `ResponseExt`, land one bounded first
> slice, and keep that work out of both the umbrella lane and the shared `fret-authoring`
> contract.

## Why this is a new lane

This should not be forced back into `imui-editor-grade-product-closure-v1` because the remaining
question is now implementation-heavy and tightly scoped.

It also should not be forced into a broader "key owner / convenience backlog" lane because the
current evidence does not justify mixing:

- item-status lifecycle vocabulary,
- key-owner semantics,
- collection proof breadth,
- and pane/menu/tab proof depth

into one folder again.

This lane is narrower than the umbrella:

- the umbrella keeps phase ordering and cross-phase status,
- this lane owns only the `ResponseExt` lifecycle vocabulary and its first bounded implementation
  slice.

## Assumptions-first baseline

### 1) `fret-authoring::Response` must stay unchanged.

- Evidence:
  - `ecosystem/fret-authoring/src/lib.rs`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/UIWRITER_RESPONSE_CONTRACT_CLOSEOUT_2026-03-29.md`
  - `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would accidentally reopen a shared contract change instead of a facade-only follow-on.

### 2) Richer lifecycle status stays in `fret-ui-kit::imui::ResponseExt`.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would blur the owner split between shared authoring contracts and ecosystem-level
    convenience vocabulary.

### 3) Existing pressable helpers already harvest transient events plus per-item/window state into `ResponseExt`.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the first slice would need a much larger runtime or control rewrite than the current evidence
    suggests.

### 4) Key ownership is a separate follow-on unless stronger evidence appears.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - this lane would grow into another generic immediate-convenience backlog.

## Goals

1. Freeze the owner split for richer immediate lifecycle status.
2. Define one small, reviewable first lifecycle vocabulary instead of adding ad hoc flags.
3. Land the first slice without widening `fret-authoring::Response` or `crates/fret-ui`.
4. Leave one demo surface, one gate package, and one evidence set for the lane.

## Non-goals

- Widening `fret-authoring::Response`.
- Widening `crates/fret-ui`.
- Solving key-owner parity in this lane.
- Solving collection proof breadth or pane/menu/tab policy depth in this lane.
- Cloning every Dear ImGui `IsItem*()` bit without repeated first-party proof.

## Initial target surface

This lane does not start from zero.
`ResponseExt` already exposes hover, click variants, drag lifecycle, context-menu request, and
nav-highlight posture.

The missing gap is the narrower lifecycle vocabulary around:

- the item just became active,
- the item just stopped being active,
- the interaction edited a value,
- and deactivation happened after an edit during the same active session.

The initial quartet is:

- `activated`
- `deactivated`
- `edited`
- `deactivated_after_edit`

Target semantics:

- `activated` reports a one-frame edge when the item enters its active or engaged state.
- `deactivated` reports a one-frame edge when that active or engaged state ends.
- `edited` reports a one-frame edge when a control commits a meaningful value mutation during the
  current frame.
- `deactivated_after_edit` reports a one-frame edge when the item deactivates after at least one
  `edited` event happened during the same active session.

Interpretation rules for the first slice:

- click-only controls may produce `activated` / `deactivated` while keeping `edited = false`,
- value-editing controls should align `edited` with existing `core.changed` evidence where
  possible instead of inventing a second meaning,
- and the lane should stay inside this quartet until repeated evidence says otherwise.

Current first landed slice:

- direct pressables: `button` and `selectable`,
- value-editing pressables: `checkbox_model`, `switch_model`, and `slider_f32_model`,
- focus-owned text entry: `input_text_model` and `textarea_model`.

## Default owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- `ResponseExt` lifecycle fields and accessors,
- transient/per-item/window-state plumbing used to derive those fields,
- and the bounded facade policy needed to keep the semantics coherent.

### `ecosystem/fret-imui`

Owns:

- focused interaction tests for the lifecycle edges,
- frame/timing expectations for the immediate frontend,
- and proof that the facade semantics remain stable across real interactions.

### `apps/fret-examples`

Owns:

- the small demo/readability proof for the lifecycle vocabulary,
- currently `apps/fret-examples/src/imui_response_signals_demo.rs` plus source-policy freeze tests.

### Not owned here

- `ecosystem/fret-authoring`
  - shared minimal response contract stays unchanged.
- `crates/fret-ui`
  - mechanism/runtime contract stays unchanged.
- key-owner/global shortcut ownership
  - still separate unless later evidence justifies another narrow follow-on.

Do not widen `crates/fret-ui` or invent a global key-owner model in this lane.

## Execution rules

1. Use the umbrella lane for phase ordering and cross-phase status.
2. Use this lane only for `ResponseExt` lifecycle vocabulary and its first bounded implementation
   slice.
3. If pressure shifts to key ownership, start a separate narrow follow-on.
4. If pressure shifts to collection/pane/menu/tab proof breadth, keep that out of this folder.
5. Every slice in this lane must name:
   - one immediate proof surface,
   - one focused gate package,
   - and one evidence set.

## Current first-open proof order

1. `apps/fret-examples/src/imui_response_signals_demo.rs`
2. `ecosystem/fret-ui-kit/src/imui/response.rs`
3. `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
4. `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
5. `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
6. `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

## Success condition

This lane succeeds when the repo can answer one narrow question cleanly:

> what is the first shipped `ResponseExt` lifecycle vocabulary for editor-grade immediate widgets,
> which layer owns it, and which tests/demos keep it stable?

That does not require a runtime rewrite.
It requires one bounded vocabulary, one owner split, and one reviewable first slice.
