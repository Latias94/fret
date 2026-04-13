# ImUi Menu/Tab Trigger Response Canonicalization v1

Status: closed historical design note
Last updated: 2026-04-13

Status note (2026-04-13): this document remains useful for the narrow cleanup rationale and target
surface, but the shipped verdict now lives in `FINAL_STATUS.md` and `WORKSTREAM.json`. Read the
compatibility-track names below as historical/deleted unless explicitly marked as retained history.

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

This lane exists because `imui-menu-tab-trigger-response-surface-v1` already landed the additive
outward response helpers, but it left a second compatibility track behind:

- `begin_menu_response[_with_options]` / `begin_submenu_response[_with_options]` /
  `tab_bar_response[_with_options]`
- plus the older `begin_menu[_with_options]` / `begin_submenu[_with_options]` /
  `tab_bar[_with_options]` wrappers

That keeps the codebase technically compatible, but it also makes the immediate-mode authoring
surface harder to teach and maintain.

## Why this is a new lane

The previous lane is already closed and now acts as the historical record for the landed helper
trigger response surface. Reopening it just to delete the compatibility track would blur two
different decisions:

1. should the surface exist at all,
2. and which public method names should remain canonical after it exists.

This lane owns only the second question.

## Assumptions-first baseline

### 1) The duplicate helper surface is still local enough to canonicalize in one slice.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-imui/src/tests/composition.rs`
  - `ecosystem/fret-imui/src/tests/interaction.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this cleanup would fan out across too many downstream examples and should be split further.

### 2) Returning richer responses from the canonical helper names is source-manageable.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - current call-site search on `begin_menu*`, `begin_submenu*`, and `tab_bar*`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would need a staged migration instead of one cleanup slice.

### 3) This remains a `fret-ui-kit::imui` facade cleanup, not a mechanism-layer change.

- Evidence:
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would need ADR/runtime-contract work instead of staying inside the ecosystem facade.

## Goals

1. Collapse helper-owned menu/submenu/tab trigger APIs onto one canonical outward response surface.
2. Remove the duplicate `*_response*` compatibility entry points if the canonical names can carry
   the richer return values cleanly.
3. Leave focused tests, demo proof, and source-policy proof on the canonical names.

## Non-goals

- Widening `fret-authoring::Response`
- Widening `crates/fret-ui`
- Reopening the “should this surface exist?” decision
- Bundling richer menu/submenu/tab policy into this lane
- Solving key ownership or shell-level tabstrip product behavior here

## Intended target surface

Canonical public helper names should be:

- `begin_menu[_with_options] -> DisclosureResponse`
- `begin_submenu[_with_options] -> DisclosureResponse`
- `tab_bar[_with_options] -> TabBarResponse`

The duplicate response-specific aliases should disappear once the canonical names carry the richer
return values.

## Owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- the canonical helper method names,
- removal of duplicate alias entry points,
- and any small response-type polish needed for the final API.

### `ecosystem/fret-imui`

Owns:

- focused behavior proof on canonical helper names,
- and regression coverage that the cleanup did not change menu/submenu/tab behavior.

### `apps/fret-examples`

Owns:

- demo/source proof on the canonical names,
- and source-policy tests that freeze the new lane and the final helper naming story.

## Success condition

This lane succeeds when the repo can teach one clean story:

> helper-owned menu/submenu/tab triggers expose one canonical outward response surface under their
> primary method names, with no duplicate alias layer left behind.
