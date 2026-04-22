# M0 Baseline Audit - 2026-04-21

Status: accepted baseline audit
Last updated: 2026-04-21

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

## Assumptions-first read

- Area: lane state
  - Assumption: this should start as a new narrow follow-on instead of reopening the umbrella or
    the closed trigger-response lanes.
  - Evidence:
    - `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
    - `docs/workstreams/imui-editor-grade-product-closure-v1/MILESTONES.md`
    - `docs/workstreams/README.md`
  - Confidence: Confident
  - Consequence if wrong: new work would blur the ownership of closed P0 lanes and make future
    reopen decisions harder.

- Area: prior menu/tab lanes
  - Assumption: the existing menu/tab lanes only settled outward response shape and canonical
    naming; they did not settle richer policy depth.
  - Evidence:
    - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
    - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
  - Confidence: Confident
  - Consequence if wrong: this lane would duplicate a problem that is already closed.

- Area: current shipped floor
  - Assumption: the current generic IMUI family still provides click-open top-level menus, click-
    opened submenus, and simple selected-model tab bars rather than richer policy.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
    - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
    - `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
  - Confidence: Confident
  - Consequence if wrong: the lane would target the wrong missing behavior.

- Area: owner split
  - Assumption: richer menubar/submenu policy may belong in generic IMUI, but richer tab overflow /
    reorder / close policy still needs an owner audit against shell/product owners.
  - Evidence:
    - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
    - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
    - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - Confidence: Likely
  - Consequence if wrong: the lane could either under-build generic IMUI or wrongly pull
    workbench-shell behavior into the shared immediate layer.

- Area: local executability
  - Assumption: this lane is a good macOS-first follow-on because it can be driven by focused
    immediate demos and tests rather than Windows mixed-DPI or Wayland validation.
  - Evidence:
    - `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
    - `apps/fret-examples/src/imui_response_signals_demo.rs`
    - `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
  - Confidence: Confident
  - Consequence if wrong: the lane would be blocked by the same platform constraints that currently
    limit the P3 runner work.

## Findings

### 1) Top-level menubar switching is still click-only

`begin_menu_with_options(...)` toggles the popup only on trigger activation:

- click or focused item-local shortcut opens/closes the menu,
- but there is no shared menubar session that switches from one open top-level menu to another on
  hover.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

### 2) Submenu depth is present, but submenu policy depth is still thin

`begin_submenu_with_options(...)` opens nested menus and exposes outward trigger responses, but it
still has no hover-switch or grace-intent behavior. The current proof floor is about click-open,
semantics, and focused trigger shortcuts, not desktop menubar feel.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

### 3) Tab bars remain intentionally simple

`tab_bar_with_options(...)` currently owns:

- selected-model normalization,
- simple trigger rendering,
- and panel switching.

It does not own overflow, scroll buttons, reordering, or close buttons.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

### 4) Shell-mounted tabstrip behavior already has a different owner story

The workbench shell lane and tabstrip proof matrix already treat richer editor shell behavior as a
separate owner question. That means this lane should not assume that every Dear ImGui-style tab
affordance belongs in the generic IMUI helper family.

Evidence:

- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`

## Execution consequence

Start this lane as an audit-first narrow follow-on with one preferred implementation order:

1. freeze whether top-level hover-switch and submenu hover-switch/grace belong in generic IMUI;
2. if yes, land that first slice with focused `fret-imui` tests and one first-party proof surface;
3. keep tab overflow/reorder/close as an owner-audit question until the shell/product split is
   explicit.
