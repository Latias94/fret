# M2 Active Menubar Mnemonic/Roving Owner Verdict - 2026-04-22

Status: landed verdict within the active lane

## Purpose

Decide whether outer-scope active-menubar posture belongs in generic
`fret-ui-kit::imui::menu_bar(...)` or should remain shell/product-owned.

This note is intentionally narrower than the already-landed generic IMUI keyboard slice:

- generic IMUI already owns trigger-local `ArrowDown` / `ArrowUp` open,
- and in-menu top-level `ArrowLeft` / `ArrowRight` switching,
- but outer-scope mnemonic activation and closed-state active-menubar choreography still needed an
  explicit owner verdict.

## Evidence reviewed

Lane docs:

- `docs/workstreams/imui-menu-tab-policy-depth-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/MILESTONES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_MENUBAR_KEYBOARD_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_MENUBAR_KEYBOARD_POSTURE_SLICE_2026-04-22.md`

Implementation and proof anchors:

- `ecosystem/fret-ui-kit/src/imui/options/menus.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret/src/in_window_menubar.rs`

Upstream comparison anchor:

- `repo-ref/imgui/imgui.cpp`

## Findings

### 1. Generic IMUI does not have the right authoring contract for mnemonic posture

`MenuBarOptions` only carries layout/testing knobs, and `BeginMenuOptions` only carries enablement,
popup options, and trigger-local `activate_shortcut`.

There is no generic IMUI contract for:

- top-level mnemonic declaration,
- platform-specific mnemonic gating,
- mnemonic underline rendering,
- or active-menubar entry/exit ownership outside the focused trigger itself.

That absence is useful evidence, not an accident:
adding outer-scope mnemonic posture here would require a new authoring surface, not just plumbing a
missing callback.

### 2. The reusable `trigger_row` primitives are necessary but not sufficient

`trigger_row` already exposes the right low-level helpers:

- `open_on_alt_mnemonic(...)`,
- `open_on_mnemonic_when_active(...)`,
- and `exit_active_on_escape_when_closed(...)`.

But those helpers assume more state than generic IMUI currently owns:

- a stable registry with mnemonic values,
- knowledge of whether current focus is still on the trigger row,
- a remembered `last_focus_before_menubar`,
- and a focus-bridge surface that can restore or arm focus outside the immediate popup subtree.

That is shell bridge state, not merely menu trigger state.

### 3. The full posture already has a first-party owner in `fret::in_window_menubar`

`in_window_menubar` already carries the complete shell-shaped contract:

- runtime menu data with `menu.mnemonic`,
- explicit `InWindowMenubarFocusHandle`,
- `last_focus_before_menubar`, `focus_is_trigger`, and `pending_focus` models,
- installed bridge handlers for Alt/mnemonic/Escape,
- top-level roving focus with typeahead on the trigger row,
- and focused tests for active mnemonic behavior when trigger descendants hold focus.

That is the right owner signal:
the repo already has one stronger product/shell surface for this behavior, so generic IMUI does not
need to grow by parity instinct alone.

### 4. Dear ImGui does not create pressure for a generic mnemonic facade here

In the local `repo-ref/imgui` snapshot, the notable `F10` handling in `imgui.cpp` is wired to
navigation context-menu requests, not a Windows-style top-level menubar mnemonic system.

So the active-menubar mnemonic story here is driven more by shell/platform expectations and Fret's
first-party in-window menubar than by a missing Dear ImGui parity surface.

## Verdict

Keep outer-scope active-menubar mnemonic/roving posture out of generic `fret-ui-kit::imui`.

Default owner:

- `ecosystem/fret::in_window_menubar`
- other shell/product menubar surfaces that can own focus bridge state and platform posture

Not the default owner:

- `ecosystem/fret-ui-kit::imui::menu_bar`
- `begin_menu_with_options(...)`
- `BeginMenuOptions` / `MenuBarOptions` surface growth just because `trigger_row` already has the
  low-level helper

## Immediate consequence for this lane

After this verdict, the remaining generic IMUI pressure is narrower:

- richer submenu intent tuning beyond the current grace corridor,
- and reverse-direction focus arbitration when top-level `ArrowLeft` / `ArrowRight` switching
  reopens an earlier sibling.

The lane no longer needs to keep "outer-scope mnemonic / roving owner" open as an unresolved
generic IMUI question.
