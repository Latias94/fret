# M2 Landed Menu Policy Floor - 2026-04-22

Purpose: record which menu/submenu policy-depth slice actually landed in generic IMUI after the
baseline audit, and keep the remaining owner questions explicit instead of widening the lane by
accident.

## Evidence reviewed

Lane docs:

- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/MILESTONES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`

Implementation and proof anchors:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub_trigger.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`

Upstream comparison anchors:

- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui.cpp`

## Findings

### 1. Top-level menubar hover-switch now belongs to the generic IMUI floor

The landed slice admits the desktop-style outcome that mattered most for the current first-party
proof:

- once a top-level menu is already open,
- hovering a sibling trigger can switch the active/open top-level menu after the trigger hover
  delay,
- and the implementation stays inside ecosystem-owned popup/menu policy state instead of widening
  `crates/fret-ui` or `fret-authoring::Response`.

This is the right owner split for the current pressure because it is still helper-owned interaction
policy, not a runtime mechanism gap.

### 2. Submenu hover-open, sibling hover-switch, and a basic grace corridor now belong to the generic IMUI floor

The landed slice also admits the first richer submenu policy step:

- pointer entry on a submenu trigger can now open the nested menu,
- hovering a sibling submenu trigger can switch the nested popup after the shared delay,
- keyed open-state tracking keeps sibling submenu triggers from leaking state into one another,
- submenu trigger policy already carries a basic grace corridor rather than instant collapse on
  every crossing,
- and submenu trigger keyboard activation still keeps its focused `activate_shortcut` behavior.

However, the lane still has not admitted:

- richer submenu-intent tuning beyond the current grace corridor,
- or broader keyboard-owner choreography beyond the existing trigger-local seams.

Those remain active follow-on questions, not implied by this landed floor.

### 3. The tab family is still intentionally thinner than Dear ImGui

`repo-ref/imgui` still has a broader tab bar story than Fret's current generic IMUI family:

- `BeginTabBar()` supports fitting-policy flags, reorderable tabs, close buttons, and tab-list
  popup behavior,
- `TabItemButton()` supports leading/trailing action tabs,
- and the demo exercises richer add/close/reorder workflows than Fret's current helper family.

The current Fret tab family still stops at:

- selected-model normalization,
- simple trigger rendering,
- and panel switching with outward trigger responses.

That difference is still intentional until the repo decides whether richer tab policy belongs in
generic IMUI or in workspace/product owners.

### 4. This landed slice should not be misread as a broader shortcut or runtime reopening

Dear ImGui also exposes `SetNextItemShortcut()` and `SetItemKeyOwner()`, but this lane did not
reopen that problem.

The current landed work:

- reuses the existing helper-local `activate_shortcut` seams,
- keeps menu/submenu policy in `fret-ui-kit::imui`,
- and relies on popup/menu policy plumbing rather than a new runtime command/key-owner surface.

That preserves the split already frozen by the separate closed key-owner lane.

## Verdict

The lane now has one shipped generic IMUI policy-depth floor:

- top-level menubar hover-switch,
- submenu hover-open,
- sibling submenu hover-switch,
- and a basic submenu grace corridor.

Keep the lane active.
Do not widen the same slice to cover richer submenu-intent tuning, roving/mnemonic posture, or
richer tab affordances without another explicit owner/proof pass.

## Immediate execution consequence

From this point forward:

1. treat `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs` as the current executable floor
   for generic menu/tab policy depth,
2. keep `apps/fret-examples/src/imui_interaction_showcase_demo.rs` and
   `apps/fret-examples/src/imui_response_signals_demo.rs` as the first-open proof surfaces for the
   shipped generic menu policy floor,
3. compare any richer tab ask against `apps/fret-examples/src/workspace_shell_demo.rs` before
   widening generic IMUI,
4. keep `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale pressure out of this lane unless a
   different narrower follow-on explicitly reopens that owner question.
