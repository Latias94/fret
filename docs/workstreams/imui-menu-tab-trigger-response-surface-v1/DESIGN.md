# ImUi Menu/Tab Trigger Response Surface v1

Status: closed execution record
Last updated: 2026-04-13

Related:

- `M0_BASELINE_AUDIT_2026-04-13.md`
- `FINAL_STATUS.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/TODO.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

This lane existed because the current `ResponseExt` lifecycle follow-on already closed its first
public response slices and then hit a different boundary:

> helper-owned menu/submenu/tab triggers already have real interaction state internally, but the
> outward API still collapses that state into `bool open` or no return value at all.

That was no longer the same question as "what lifecycle vocabulary should `ResponseExt` expose?"
It was a separate outward-surface decision, and this lane now records the landed answer.

## Why this is a new lane

This should not be forced back into `imui-response-status-lifecycle-v1` because that lane already
froze its owner split around public response surfaces that exist today (`ResponseExt` and
`ComboResponse`) and explicitly deferred helper-owned trigger response work.

It also should not be widened into generic menu/tab policy depth because the pressure was
much narrower:

- do helper-owned triggers need an outward response surface at all,
- if yes, what is the smallest API budget,
- and how do we keep that separate from richer menu-bar/submenu/tab policy?

## Assumptions-first baseline

### 1) Legacy compatibility helpers keep their existing `bool open` / no-return posture.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the landed surface would silently break existing helper call sites instead of remaining
    additive.

### 2) Additive response entry points are now the narrow surface budget for this lane.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would still be underspecified and authors would not know which helper-level response
    surface is canonical.

### 3) The internal trigger implementations carried enough state to support an additive outward surface.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the landed surface work would have required a larger helper rewrite than the current evidence
    suggests.

### 4) Richer menu-bar/submenu/tab policy depth still belongs outside this lane.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would become another generic `imui` backlog instead of a narrow response-surface
    decision.

## Goals

1. Land the helper-owned outward response verdict explicitly.
2. Keep the landed surface in `ecosystem/fret-ui-kit::imui` without widening
   `fret-authoring::Response` or `crates/fret-ui`.
3. Leave one current-behavior floor and one source-policy gate so the verdict stays executable.

## Non-goals

- Widening `fret-authoring::Response`.
- Widening `crates/fret-ui`.
- Bundling richer menu-bar/submenu/tab policy into this lane.
- Solving key-owner/global shortcut ownership here.
- Reopening the shell-level tab-strip/workbench product lane.

## Landed surface

This lane closes with additive helper-level response entry points:

- `begin_menu_response[_with_options]`
- `begin_submenu_response[_with_options]`
- `tab_bar_response[_with_options]`

Compatibility wrappers remain:

- `begin_menu_with_options` and `begin_submenu_with_options` still return `bool open`.
- `tab_bar_with_options` still returns no outward value and simply ignores the richer response.

Landed response budget:

- menu/submenu response uses `DisclosureResponse`,
- tab response uses `TabBarResponse` + `TabTriggerResponse`,
- and richer menu/tab policy stays out of scope.

This keeps legacy call sites stable while giving immediate-mode authors an opt-in surface for
trigger `clicked` / lifecycle / open / selected-change queries.

## Default owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- any future outward trigger response surface,
- the landed additive response entry points,
- helper-local bridging between existing internal trigger state and the public facade,
- and the narrow API shape budget for these helpers.

### `ecosystem/fret-imui`

Owns:

- the current behavior floor for menu/submenu/tab helper behavior,
- focused interaction tests for the landed trigger response surface,
- and regression proof that helper ergonomics do not regress after the surface landed.

### `apps/fret-examples`

Owns:

- source-policy freeze tests for the follow-on split and landed contract,
- and first-open demo/source proof for the additive outward trigger surface.

### Not owned here

- `ecosystem/fret-authoring`
  - shared minimal response contract stays unchanged.
- `crates/fret-ui`
  - runtime contract stays unchanged.
- richer menu-bar/submenu/tab policy
  - still separate unless later evidence proves the response-surface decision is insufficient.

## Execution rules

1. Keep this lane about helper-owned trigger response surfaces only.
2. Preserve compatibility wrappers while exposing richer response entry points separately.
3. Keep the landed slice facade-only.
4. Do not bundle richer menu-bar/submenu/tab policy into this lane.

## Current first-open proof order

1. `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
2. `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
3. `ecosystem/fret-ui-kit/src/imui.rs`
4. `ecosystem/fret-imui/src/tests/interaction.rs`
5. `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
6. `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

## Success condition

This lane succeeded once the repo could answer one narrow question cleanly:

> should helper-owned menu/submenu/tab triggers stay model-driven only, or should they grow a
> narrow outward response surface, and what evidence keeps that decision stable?

The landed answer is "add one narrow outward facade while preserving the old helper wrappers."
The important part is that it became explicit, reviewable, and separate from the already-landed
`ResponseExt` lifecycle vocabulary lane and from broader menu/tab policy depth.
