# ImUi Menu/Tab Trigger Response Surface v1 - M0 Baseline Audit (2026-04-13)

Status: active status note
Last updated: 2026-04-13

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

## Why this audit exists

The active lifecycle lane reached a hard edge:

- public response surfaces like `menu_item_with_options`, `combo_with_options`, and
  `combo_model_with_options` could absorb lifecycle semantics directly,
- but helper-owned menu/submenu/tab triggers do not expose trigger response state outward today.

This audit records why that should now be treated as its own narrow follow-on instead of another
TODO inside the lifecycle lane.

## Assumptions-first read

### 1) Menu helper triggers already produce richer internal state than their outward API shows.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the repo would be splitting a follow-on for a surface that is not actually internally rich
    enough to surface later.

### 2) Tab helper triggers already build per-item trigger responses internally but erase them before the public boundary.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - tab trigger outward response would still be a purely hypothetical gap rather than an exposed
    internal/public mismatch.

### 3) The remaining decision is about outward surface, not about `ResponseExt` vocabulary.

- Evidence:
  - `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/TODO.md`
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the repo would blur the already-landed lifecycle vocabulary lane with a second API-shape lane.

### 4) Richer menu/tab policy should stay out of this follow-on.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - this follow-on would balloon into a full menu/tab policy redesign.

## Findings

### 1) `begin_menu_with_options` and `begin_submenu_with_options` collapse trigger detail into `bool open`

The current helper path builds a trigger `ResponseExt`, uses `trigger.clicked()` to toggle popup
state, and then returns only the post-build `bool open`.

That means authors cannot observe:

- trigger `clicked`,
- trigger `activated` / `deactivated`,
- trigger `focused` / `hovered`,
- or future trigger lifecycle additions

without rebuilding the helper behavior locally.

### 2) `tab_bar_with_options` already computes trigger responses but exposes none of them

`tab_family_controls.rs` already builds `BuiltTabTrigger { element, response }`, tracks selected
trigger identity, and switches the selected model.

The public API still returns `()`.
That is a classic "internal evidence exists, public decision not yet frozen" shape.

### 3) This is a better fit for a narrow API-shape lane than for the lifecycle lane

The lifecycle lane already answered:

- where richer immediate response vocabulary lives,
- what the first lifecycle quartet is,
- and how existing public response surfaces should expose it.

What remains here is different:

- should helper-owned triggers expose anything outward,
- if yes, what should the wrapper look like,
- and what proof surface should teach it?

### 4) The current behavior floor is already executable

The repo already has focused helper behavior tests around:

- top-level menu open/close,
- submenu open/expanded behavior,
- tab selection/panel switching,
- and tab activation shortcuts.

That makes this a good follow-on because the lane can start from executable behavior rather than
from speculative API design.

## Execution consequence

Use `imui-menu-tab-trigger-response-surface-v1` as the narrow follow-on for this question.

From this note forward:

1. keep `imui-response-status-lifecycle-v1` focused on public response surfaces that already exist,
2. keep richer menu/tab policy in the umbrella/parity audit unless fresh evidence says otherwise,
3. and treat helper-owned trigger response as a separate surface decision that may close either on
   a no-new-API verdict or on one narrow facade addition.
