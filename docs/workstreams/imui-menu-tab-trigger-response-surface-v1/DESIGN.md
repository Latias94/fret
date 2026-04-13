# ImUi Menu/Tab Trigger Response Surface v1

Status: active execution lane
Last updated: 2026-04-13

Related:

- `M0_BASELINE_AUDIT_2026-04-13.md`
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

This lane exists because the current `ResponseExt` lifecycle follow-on already closed its first
public response slices and then hit a different boundary:

> helper-owned menu/submenu/tab triggers already have real interaction state internally, but the
> outward API still collapses that state into `bool open` or no return value at all.

That is no longer the same question as "what lifecycle vocabulary should `ResponseExt` expose?"
It is a separate outward-surface decision.

## Why this is a new lane

This should not be forced back into `imui-response-status-lifecycle-v1` because that lane already
froze its owner split around public response surfaces that exist today (`ResponseExt` and
`ComboResponse`) and explicitly deferred helper-owned trigger response work.

It also should not be widened into generic menu/tab policy depth because the current pressure is
much narrower:

- do helper-owned triggers need an outward response surface at all,
- if yes, what is the smallest API budget,
- and how do we keep that separate from richer menu-bar/submenu/tab policy?

## Assumptions-first baseline

### 1) `begin_menu_with_options` and `begin_submenu_with_options` currently return only `bool open`.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would be solving a problem that already has a public response surface.

### 2) `tab_bar_with_options` currently exposes no outward trigger response surface.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - a tab-specific follow-on would be unnecessary because authors could already observe trigger
    state directly.

### 3) The internal trigger implementations already carry enough state to support a future outward surface.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the eventual surface work would require a larger helper rewrite than the current evidence
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

1. Decide whether helper-owned menu/submenu/tab triggers need outward response surfaces or a
   no-new-API verdict.
2. If a new surface is justified, keep it in `ecosystem/fret-ui-kit::imui` without widening
   `fret-authoring::Response` or `crates/fret-ui`.
3. Leave one current-behavior floor and one source-policy gate so the decision stays executable.

## Non-goals

- Widening `fret-authoring::Response`.
- Widening `crates/fret-ui`.
- Bundling richer menu-bar/submenu/tab policy into this lane.
- Solving key-owner/global shortcut ownership here.
- Reopening the shell-level tab-strip/workbench product lane.

## Current target surfaces

This lane starts from helper-owned trigger seams whose outward API is smaller than their internal
interaction state:

- `begin_menu_with_options`
- `begin_submenu_with_options`
- `tab_bar_with_options`
- `begin_tab_item_with_options`

Current outward posture:

- `begin_menu_with_options` and `begin_submenu_with_options` currently return only `bool open`.
- `tab_bar_with_options` currently exposes no outward trigger response surface.

Current candidate outcomes:

- no-new-API verdict:
  keep `bool open` / no return value and document that authors should observe open/selected state
  through explicit models instead of trigger responses;
- narrow outward response surface:
  add helper-level response wrappers that reuse `ResponseExt` internally while keeping menu/tab
  policy depth and collection policy outside the surface.

Do not freeze concrete type names or field names in this lane until the decision is clearer.

## Default owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- any future outward trigger response surface,
- helper-local bridging between existing internal trigger state and the public facade,
- and the narrow API shape budget for these helpers.

### `ecosystem/fret-imui`

Owns:

- the current behavior floor for menu/submenu/tab helper behavior,
- focused interaction tests for any eventual trigger response surface,
- and regression proof that helper ergonomics do not regress while the decision stays open.

### `apps/fret-examples`

Owns:

- source-policy freeze tests for the follow-on split,
- and any first-open demo/source proof if a new outward trigger surface lands.

### Not owned here

- `ecosystem/fret-authoring`
  - shared minimal response contract stays unchanged.
- `crates/fret-ui`
  - runtime contract stays unchanged.
- richer menu-bar/submenu/tab policy
  - still separate unless later evidence proves the response-surface decision is insufficient.

## Execution rules

1. Keep this lane about helper-owned trigger response surfaces only.
2. Reuse current behavior-floor tests before inventing new surface proof.
3. If the result is "no new API," close this lane explicitly instead of leaving a vague TODO.
4. If the result is "new outward surface," keep the first slice narrow and facade-only.
5. Do not bundle richer menu-bar/submenu/tab policy into this lane.

## Current first-open proof order

1. `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
2. `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
3. `ecosystem/fret-ui-kit/src/imui.rs`
4. `ecosystem/fret-imui/src/tests/interaction.rs`
5. `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
6. `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

## Success condition

This lane succeeds when the repo can answer one narrow question cleanly:

> should helper-owned menu/submenu/tab triggers stay model-driven only, or should they grow a
> narrow outward response surface, and what evidence keeps that decision stable?

That answer may be "no new API."
The important part is that it becomes explicit, reviewable, and separate from the already-landed
`ResponseExt` lifecycle vocabulary lane.
