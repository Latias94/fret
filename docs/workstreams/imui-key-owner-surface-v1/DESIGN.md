# ImUi Key Owner Surface v1

Status: closed closeout reference
Last updated: 2026-04-21

Status note (2026-04-21): this document remains the lane-opening rationale. The shipped verdict now
lives in `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md` and `CLOSEOUT_AUDIT_2026-04-21.md`. References
below to implementation-heavy key-owner surface work should be read as opening-state rationale
rather than an active execution queue.

Related:

- `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
- `CLOSEOUT_AUDIT_2026-04-21.md`
- `M0_BASELINE_AUDIT_2026-04-21.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

This lane exists because the maintenance umbrella already narrowed the remaining P0 backlog and the
collection/pane proof lane just closed without consuming the still-separate key-owner pressure.

The smallest credible follow-on is now:

> freeze the immediate key-owner / item-local shortcut ownership problem as its own lane,
> decide whether the current focused `activate_shortcut` and command-metadata seams are enough,
> and land any future additive surface without reopening `crates/fret-ui`, lifecycle vocabulary,
> collection/pane proof breadth, or richer menu/tab policy.

## Why this is a new lane

This should not be forced back into `imui-editor-grade-product-closure-v1` because the remaining
question is now implementation-heavy and narrowly about key-owner ergonomics.

It also should not be mixed with already-separated topics:

- `imui-response-status-lifecycle-v1` already closed the first `ResponseExt` lifecycle quartet.
- `imui-collection-pane-proof-v1` already closed the collection/pane proof pair.
- menu/submenu/tab policy depth already belongs to the separate trigger-response / policy chain.
- runner/backend multi-window parity still belongs to the active docking lane.

This lane is narrower than the umbrella:

- the umbrella keeps phase ordering and cross-phase status,
- this lane owns only the remaining immediate key-owner / item-local shortcut ownership question
  and the first bounded implementation slice that may follow from it.

## Assumptions-first baseline

### 1) Focused item-local shortcut coverage exists, but the broader key-owner surface is still open.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `ecosystem/fret-imui/src/tests/interaction.rs`
  - `ecosystem/fret-imui/src/tests/models.rs`
  - `ecosystem/fret-imui/src/tests/popup_hover.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would start from the wrong baseline and either overstate the gap or miss an already
    sufficient shipped verdict.

### 2) `crates/fret-ui` must remain unchanged unless stronger ADR-backed evidence appears.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would drift into runtime widening instead of ecosystem-level shortcut ownership proof.

### 3) Global keymap / command routing semantics remain fixed input, not negotiable scope here.

- Evidence:
  - `docs/adr/0020-focus-and-command-routing.md`
  - `docs/adr/0023-command-metadata-menus-and-palette.md`
  - `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would blur helper ergonomics with host/runtime shortcut arbitration and become too
    large to review.

### 4) The current proof is mostly test-first rather than demo-first.

- Evidence:
  - `apps/fret-examples/src/imui_response_signals_demo.rs`
  - `ecosystem/fret-imui/src/tests/interaction.rs`
  - `ecosystem/fret-imui/src/tests/models.rs`
  - `ecosystem/fret-imui/src/tests/popup_hover.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would choose the wrong first-open repro and overspecify a demo requirement too early.

### 5) Adjacent lanes stay separate unless fresh evidence proves the owner split wrong.

- Evidence:
  - `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - this folder would become another generic immediate-convenience backlog instead of a narrow
    follow-on.

## Goals

1. Freeze the owner split for immediate key-owner / item-local shortcut ownership work.
2. Decide whether Fret needs any additive immediate equivalent to
   `SetNextItemShortcut()` / `SetItemKeyOwner()` beyond the current helper-local
   `activate_shortcut` seam.
3. Keep the first slice bounded to one repro set, one gate package, and one evidence set.
4. Leave clear follow-on policy if the answer is "not yet" or "separate owner."

## Non-goals

- Widening `crates/fret-ui`.
- Redesigning global keymap or command-routing semantics.
- Reopening `ResponseExt` lifecycle vocabulary.
- Reopening collection/pane proof breadth.
- Reopening broader menu/submenu/tab policy depth.
- Reopening runner/backend multi-window parity.

## Initial target surface

This lane does not start from zero.
Fret already has a focused immediate shortcut surface:

- helper-local `activate_shortcut` options on the current immediate control families,
- `shortcut_repeat` as an explicit opt-in seam,
- `menu_item_command[_with_options]` and `button_command[_with_options]` for command metadata,
- and interaction tests proving focus-scoped local activation across buttons, selectables,
  checkboxes, switches, menus, submenus, tabs, and combo triggers.

The missing gap is the narrower question around key ownership depth:

1. whether the current helper-local `activate_shortcut` seam is sufficient,
2. whether Fret needs an additive immediate equivalent to
   `SetNextItemShortcut()` / `SetItemKeyOwner()`,
3. whether a broader item-local shortcut registration seam is justified beyond the current
   helper-specific option fields,
4. and what first-party proof surface should exist beyond the current test-heavy floor.

The lane starts from the current first-open repro set:

- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`

`imui_response_signals_demo` is the current proof/contract surface, while the `fret-imui` test
suite is the current executable behavior floor.
M1 will decide whether that is enough or whether a stronger first-party editor-grade proof surface
must be promoted.

## Default owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- additive helper-local shortcut or key-owner facade seams,
- focused option-shape decisions above the fixed runtime command/keymap contract,
- and the smallest helper growth directly justified by first-party proof.

### `ecosystem/fret-imui`

Owns:

- focused interaction proof for shortcut scoping and local activation behavior,
- the immediate-layer behavior floor for buttons/selectables/menus/tabs/combos,
- and proof that any future key-owner surface still respects focus-local behavior.

### `apps/fret-examples`

Owns:

- first-party proof/contract demos,
- source-policy tests that keep the lane explicit,
- and any future editor-grade proof surface promoted by this lane.

### Not owned here

- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- `crates/fret-app` and `crates/fret-runtime`
  - global keymap, command routing, and shortcut arbitration remain fixed input here.
- `ResponseExt` lifecycle vocabulary
  - already closed in `imui-response-status-lifecycle-v1`.
- collection/pane proof breadth
  - already closed in `imui-collection-pane-proof-v1`.
- broader menu/submenu/tab policy
  - remains on the trigger-response / policy follow-on chain.
- runner/backend multi-window parity
  - remains owned by `docs/workstreams/docking-multiwindow-imgui-parity/`.

## Execution rules

1. Use the umbrella lane for cross-phase status and follow-on policy.
2. Start from proof of the current focused shortcut behavior before proposing new key-owner API
   growth.
3. Do not reopen `ResponseExt` lifecycle vocabulary, collection/pane proof breadth, or richer
   menu/tab policy in this lane.
4. If pressure shifts to runtime keymap/IME arbitration, start or resume a different ADR-backed
   lane instead of widening this folder.
