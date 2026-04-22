# M2 Menubar Keyboard Audit - 2026-04-22

Purpose: record what the current generic IMUI top-level menu architecture actually permits for
menubar keyboard posture, and prevent the lane from widening on the false assumption that the
existing `trigger_row` primitives can be wired in without another owner/refactor pass.

## Evidence reviewed

Lane docs:

- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/MILESTONES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Implementation and proof anchors:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret/src/in_window_menubar.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

Local repro command reviewed during this audit:

- `cargo nextest run -p fret-imui --no-fail-fast`

Upstream comparison anchors:

- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_demo.cpp`

## Findings

### 1. The missing generic IMUI keyboard depth is not a missing primitive

`ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs` already carries the important
desktop-style menubar policies the lane was still questioning:

- top-level left/right switching while a menu is active,
- active-menubar mnemonic opening,
- and closed-state `Escape` exit posture for Windows/Linux-style in-window menubars.

So the remaining gap is not "invent another mechanism in `crates/*`" and it is not proof that the
runtime contract is missing.

### 2. Generic IMUI top-level menus still have a different owner contract than popup/context menus

The current top-level `begin_menu_with_options(...)` path does not reuse
`begin_popup_menu_with_options(...)`.

Instead, it still:

- coordinates top-level ownership through `ImUiMenubarPolicyState::open_menu`,
- opens/closes individual popup stores with `popup_open_model(...)` and `open_popup_at(...)`,
- builds popup children with `build_popup_menu(...)`,
- and then mounts the returned children directly with `ui.add(...)`.

By contrast, `begin_popup_menu_with_options(...)` is the path that goes on to install the
`dismissible_menu_request_with_modal(...)` root plus `MenuInitialFocusTargets`, including
`keyboard_entry_focus(built.first_item)`.

Inference from those anchors:
top-level IMUI menus do not currently share the same popup-root / autofocus / dismiss-contract as
popup/context-menu surfaces.

### 3. The current top-level IMUI menu still behaves like a trigger-owned popup, not a content-owned menu root

During the local focused repro, the first open top-level menu kept focus on the top-level trigger
instead of moving focus into the first menu item.

That result matches the code shape above:

- top-level `begin_menu_with_options(...)` never routes through the popup-menu helper that installs
  initial keyboard focus targets,
- so trigger-focused key handling and popup-content-focused key handling are still materially
  different surfaces in generic IMUI today.

This matters because the existing `trigger_row` horizontal-switch helper used by
`in_window_menubar` assumes a different posture:

- top-level state is driven directly by per-trigger `open: Model<bool>`,
- and focused menu content can participate in the switching contract.

Generic IMUI is not there yet.

### 4. `in_window_menubar` proves the missing piece is bridge/refactor work, not another policy guess

`ecosystem/fret/src/in_window_menubar.rs` already demonstrates the fuller shape that makes those
keyboard policies credible:

- explicit menubar focus bridge state,
- outer-scope command/key handling,
- trigger registry ownership,
- and a top-level menu surface built around a single, consistent menu-root contract.

That is a stronger signal than "just add one more key handler to IMUI":
if this lane wants richer generic menubar keyboard posture, it first needs either

1. a refactor that moves top-level `begin_menu` onto the same popup-root/focus contract as the
   other menu surfaces, or
2. an explicit verdict that this richer keyboard choreography remains outside generic IMUI.

## Verdict

Do not land generic IMUI menubar roving/mnemonic growth by directly plumbing the existing
`trigger_row` keyboard helpers into the current `begin_menu_with_options(...)` surface.

Keep the lane active, but narrow the remaining question further:

- the next justified slice is now a top-level-menu focus/overlay-owner decision,
- not "add more keyboard policy because the primitive exists",
- and not a runtime-layer reopening.

## Immediate execution consequence

From this point forward:

1. treat `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs` vs
   `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs` as the key architectural seam for this lane,
2. require any future generic IMUI menubar keyboard change to name which contract it is using:
   current trigger-owned top-level popup, or a refactored popup-root/menu-root contract,
3. keep Windows/Linux-style outer-scope mnemonic activation (`Alt` / `F10` posture) explicitly
   out of "easy plumbing" territory until a generic IMUI bridge surface exists,
4. prefer a narrow follow-on implementation that first unifies top-level menu focus/overlay
   ownership, or else close the lane on a no-new-generic-surface verdict for richer menubar
   keyboard choreography.
