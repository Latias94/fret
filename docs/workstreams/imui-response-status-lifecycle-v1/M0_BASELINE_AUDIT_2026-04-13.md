# M0 Baseline Audit - 2026-04-13

Status: active baseline note
Last updated: 2026-04-13

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-authoring/src/lib.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

## Why this note exists

The umbrella immediate-mode product-closure lane already narrowed the remaining P0 backlog and
explicitly warned that implementation-heavy immediate convenience work should split into narrower
follow-ons.

This note records the assumptions-first baseline for the first such follow-on, so the new lane does
not reopen old questions about runtime widening, key ownership, or generic helper growth.

## Assumptions-first read

### 1) The shared response contract is already intentionally small.

- Evidence:
  - `ecosystem/fret-authoring/src/lib.rs`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/UIWRITER_RESPONSE_CONTRACT_CLOSEOUT_2026-03-29.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would start by changing the wrong contract owner.

### 2) The current gap is about lifecycle vocabulary, not about missing base click/hover/drag plumbing.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `apps/fret-examples/src/imui_response_signals_demo.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would solve the wrong problem and keep accumulating unrelated edge flags.

### 3) Existing control helpers already show a reusable harvesting pattern.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the first implementation slice would be much broader than the docs currently justify.

### 4) Keep key ownership out of this lane.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the new follow-on would blur into a second generic immediate-convenience backlog.

## Findings

### 1) The shared/core boundary is already clear and should not move here

`fret-authoring::Response` still owns only the small shared core:

- `hovered`
- `pressed`
- `focused`
- `clicked`
- `changed`
- `rect`

That is already guarded by contract-level closeout notes and compile smoke.
This lane should treat that as fixed input rather than as a negotiation surface.

### 2) `ResponseExt` already has the correct storage posture for richer status

`ResponseExt` is already where the repo keeps richer immediate response semantics:

- click variants,
- long-press and holding state,
- drag lifecycle,
- hover query policy,
- nav-highlight posture,
- context-menu request and anchor.

Existing pressable helpers already harvest transient events plus per-item/window state into
`ResponseExt`.
That means the first lifecycle slice can likely reuse the same pattern instead of introducing a new
shared response type or a second state channel.

### 3) The parity gap is now narrower than the old docs implied

The current parity audit no longer points to a broad missing immediate runtime.
It points to a bounded missing vocabulary around activation/deactivation/edit lifecycle.

In other words:

- the repo already knows how to report many useful item-local edges,
- but it still lacks the Dear ImGui-style status readout that tells authors whether an item just
  became active, just stopped being active, or was edited before deactivation.

### 4) The current proof surface is present but incomplete

`apps/fret-examples/src/imui_response_signals_demo.rs` is already the natural proof/demo surface
for this lane, but it currently demonstrates only the older response set:

- click variants,
- long-press/holding,
- drag lifecycle,
- context-menu reporting.

That makes it the right first-open demo, but not yet sufficient evidence for the missing lifecycle
quartet.

### 5) Keep key ownership out of this lane

The umbrella lane already narrowed the remaining P0 backlog into separate pressures:

- collection proof breadth,
- child-region/menu/tab depth,
- item-status lifecycle vocabulary,
- eventual key-owner surface.

This follow-on should own only the third item.
If future work starts needing key-owner semantics, it should split again rather than silently
growing here.

## Execution consequence

Use `imui-response-status-lifecycle-v1` as the active narrow P0 follow-on for the current
`ResponseExt` lifecycle gap.

From this note forward:

1. treat the shared `fret-authoring::Response` boundary as fixed for this lane,
2. treat existing `ResponseExt` transients/state plumbing as the default implementation posture,
3. focus M1 on `activated`, `deactivated`, `edited`, and `deactivated_after_edit`,
4. and split again instead of widening this folder if the work turns into key ownership or broad
   proof-surface expansion.
