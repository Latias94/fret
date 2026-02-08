# OS Menubar Workstream (Living Tracker)

This is a living implementation tracker for native OS menubar integration and user-customizable
menu bars. It complements (but does not replace) ADRs:

- ADR 0183 defines the integration seam: `Effect::SetMenuBar` (`docs/adr/0183-os-menubar-effect-setmenubar.md`).
- ADR 0023 defines the data-only menu model and how it relates to commands/keymap.

This tracker is organized as MVP milestones with acceptance criteria and evidence anchors. The goal
is to keep cross-crate seams stable while we iterate on platform specifics and UX polish.

## Reference Model (Zed / GPUI)

Zed is the primary upstream reference for editor-grade UX. In particular:

- App-level API: `App::set_menus` delegates to the platform layer with a `Keymap` reference.
  - Evidence: `repo-ref/zed/crates/gpui/src/app.rs` (`set_menus`).
- macOS implementation builds `NSMenu` and resolves key equivalents via `Keymap::bindings_for_action`.
  - Evidence: `repo-ref/zed/crates/gpui/src/platform/mac/platform.rs` (`create_menu_item`).
- Zed intentionally prefers "earlier" bindings for menu display in some cases (see the inline comment
  referencing a Zed issue). The key takeaway: shortcut display is a UX decision, not strictly “the
  effective binding in the current focus context”.
  - Evidence: same file, `create_menu_item` comment.
- Zed models system submenus on macOS (e.g. Services) explicitly.
  - Evidence: `repo-ref/zed/crates/gpui/examples/set_menus.rs` (`MenuItem::os_submenu("Services", ...)`).

We should mirror the spirit:

- Keep menus data-only, and map them in runners.
- Resolve shortcut labels using the same keymap model as the rest of the app.
- Keep enable/disable gating best-effort and context-driven, but do not let it become a new runtime
  policy surface in `fret-ui`.

## Scope / Non-goals

In scope:

- Native runner mappings for OS menubars (Windows and macOS).
- Shortcut label rendering via keymap reverse lookup.
- Best-effort enable/disable gating via `when` + an app-provided window-scoped `InputContext` snapshot.
- File-backed customization: layered `menubar.json` (project + user), hot reload.
- User-selectable presentation modes (OS menubar vs in-window menubar), without forking the menu data model.

Non-goals:

- Replacing in-window menus/context menus/command palette surfaces (ecosystem-owned).
- Standardizing every native menu feature up-front (checked/radio roles, Services, etc.).
- Turning menu composition/customization into a full plugin API before the workspace/plugin seams are locked.

## Current Status (2026-01)

Implemented (Windows-first baseline):

- `Effect::SetMenuBar { window, menu_bar }` contract.
- Windows native menu bar mapping (Win32 `HMENU` + `WM_COMMAND`).
- macOS native menu bar mapping (`NSMenu`).
- Window-scoped `InputContext` snapshots for runner-side gating (including post-dispatch publishing so
  focus/modal changes are reflected without waiting for paint).
- Window-scoped per-command enable overrides (optional; `WindowCommandEnabledService`).
- Window-scoped command availability snapshots for non-focus gating (v1: Undo/Redo).
- Layered `menubar.json` loading + hot reload.
- Patch-style customization ops (submenu paths + selectors for non-command items).

Known gaps:

- Linux OS menubar integration is deferred (ADR 0184; in-window fallback by default).
- Richer menu semantics are not standardized yet (checked/radio/native roles).
- macOS standard menu roles/system menus are only partially standardized (track in ADR 0185).
  - Current:
    - `MenuRole` hooks exist (Window/App/Help), Services system menu is supported, and standard edit
      selectors can be hinted via `OsAction` (Cut/Copy/Paste/SelectAll/Undo/Redo).
    - A minimal App menu baseline can be authored as commands (About/Preferences/Services/Hide/Hide Others/Show All/Quit),
      and the golden path (`fret-bootstrap`) handles `app.about`/`app.quit`/`app.hide*` by emitting platform effects.
  - Remaining: finalize macOS App menu conventions (exact wording + ordering, and what stays command-driven vs runner-native).
  - Preferences remains app-owned (ADR 0187); golden-path apps can route it to an in-app settings UI.

## MVP Milestones

### MVP 0: OS Menubar Seam + Windows Mapping (Done)

Goal: ship a Windows OS menubar that dispatches commands through effects, shows shortcuts from the
keymap, and disables items best-effort based on `when`.

Acceptance:

- [x] A menu item selection produces the same command dispatch path as other UI interactions
  (no direct re-entrant calls into app handlers).
- [x] Shortcut labels are derived from keymap reverse lookup for the current platform.
- [x] `when` gating is applied using a window-scoped `InputContext` snapshot.
- [x] Menu state refresh happens when the menu is opened (avoid per-frame native API spam).
- [x] Multi-window behavior: `window: None` sets the default menu for current + future windows.

Evidence anchors:

- Contract: `crates/fret-runtime/src/effect.rs` (`Effect::SetMenuBar`).
- Windows mapping: `crates/fret-launch/src/runner/desktop/windows_menu.rs`.
- Runner handling + inheritance: `crates/fret-launch/src/runner/desktop/mod.rs`.
- Window-scoped input snapshots: `crates/fret-runtime/src/window_input_context.rs`.
- Command availability seam (Undo/Redo, Router): `crates/fret-runtime/src/window_command_availability.rs` + `crates/fret-runtime/src/when_expr.rs` (`edit.can_undo`, `edit.can_redo`, `router.can_back`, `router.can_forward`).

### MVP 1: User-Customizable `menubar.json` (Done)

Goal: allow Unity-style "user can edit menus" without binding apps to a widget kit, using layered
configuration files aligned with ADR 0014.

Deliverables:

- [x] Layered config resolution (project `.fret/menubar.json` + user `menubar.json`).
- [x] Replace mode (full explicit menu definition).
- [x] Patch mode (small ops applied on top of the app baseline menu).
- [x] Patch targets support submenu paths (e.g. `["File", "Recent"]`).
- [x] Patch ops can target non-command items via selectors (e.g. submenu by title).
- [x] Hot reload via config watcher (polling-based for now).
- [x] CLI template generation for quick onboarding (`fretboard config menubar`).
- [x] `menu_bar_version: 2` supports `MenuRole` and `SystemMenuType` (ADR 0185).

Acceptance:

- [x] Invalid config does not crash; it reports a reload error status surfaced to the UI (demo toast).
- [x] Patch ops are stable and versioned (`menu_bar_version`).

Evidence anchors:

- Config format + ops: `crates/fret-runtime/src/menu.rs`.
- Layered loading: `crates/fret-app/src/config_files.rs` (`load_layered_menu_bar`).
- Apply baseline+overlay: `crates/fret-app/src/menu_bar.rs` (`apply_layered_menu_bar`).
- Hot reload wiring: `crates/fret-app/src/config_watcher.rs`.
- Demo: `apps/fret-ui-gallery/src/driver.rs`.
- CLI template: `apps/fretboard/src/config.rs`.

### MVP 2: Shortcut Display Policy (Zed-aligned) (Done)

Problem: "shortcut label for an action" is not always "the binding that would fire in the current
focus context". Zed explicitly chooses a stable display heuristic on macOS.

Goal: define and test a shortcut-display policy that is:

- stable (does not flicker as focus moves),
- compatible with layered keymaps and `when` predicates,
- consistent across OS menubar, in-window menu surfaces, and command palette.

Policy (inspired by Zed):

- Evaluate bindings against a small, deterministic set of "default" `InputContext` variants derived
  from a base context:
  - non-modal + not text input
  - non-modal + text input
  - modal + not text input
  - modal + text input
- Rank candidates by:
  1. first matching default context (earlier wins),
  2. shorter sequences (single-chord preferred),
  3. later-defined bindings (user/project overrides preferred).
- Keep enable/disable gating separate: it should use the live per-window `InputContext` snapshot.

Acceptance:

- [x] Shortcut label is stable while navigating focus within a window.
- [x] Label is consistent between OS menubar and command palette for the same `CommandId`.
- [x] Changes to keymap hot-reload update labels on next menu-open without restarting.

Reference anchors:

- Zed menu item label selection: `repo-ref/zed/crates/gpui/src/platform/mac/platform.rs` (`bindings_for_action` selection comment).

Evidence anchors:

- Display policy implementation: `crates/fret-runtime/src/keymap.rs` (`display_shortcut_for_command_sequence`).
- Windows OS menubar uses display policy: `crates/fret-launch/src/runner/desktop/windows_menu.rs`.
- Command palette uses display policy: `ecosystem/fret-ui-shadcn/src/command.rs`.
- In-window menubar bridge uses the same display policy and is shadcn-free (built from `fret-ui-kit` primitives): `ecosystem/fret-kit/src/workspace_menu.rs`.

### MVP 2.5: Menu Bar Presentation Modes (OS vs In-window) (Done)

Problem: Fret targets editor-grade UX, but also aims to be a general-purpose UI framework. Some apps
want a native OS menubar; others want fully custom window chrome (client-side menu) for consistent
visual design and cross-platform behavior.

Goal: allow users/apps to choose whether menus show up as an OS menubar and/or an in-window menubar,
while still authoring a single data-only `MenuBar` and keeping shortcut display + enablement gating
consistent.

Deliverables:

- [x] Strongly typed settings keys for menu bar presentation.
- [x] Default `auto` behavior:
  - Windows/macOS: OS menubar on, in-window off
  - Linux/Web: OS menubar off, in-window on
- [x] OS menubar can be disabled without leaving stale native menus (clear via empty menu bar).
- [x] In-window menubar rendering no longer requires shadcn; it is an ecosystem-owned recipe surface that can be restyled/replaced without changing `MenuBar`.

Evidence anchors:

- Settings model: `crates/fret-app/src/settings.rs` (`SettingsFileV1.menu_bar`).
- OS publish/clear helper: `crates/fret-app/src/menu_bar.rs` (`sync_os_menu_bar`).
- In-window fallback uses the effective layered menu: `apps/fret-ui-gallery/src/driver.rs` (`effective_menu_bar` + `should_render_in_window_menu_bar`).

### MVP 2.6: In-window Menubar Keyboard Focus (F10) (Done)

Problem: Editor-grade UX expects a "focus the menu bar" command (Windows convention: F10) that works
consistently across in-window menubars and respects keymap + command enablement.

Goal: add a single command (`focus.menu_bar`) that in-window shells can handle to move focus to the
first enabled menubar trigger (or the active trigger when closing an open menu).

Acceptance:

- [x] Default keybinding on Windows/Linux: `F10` (disabled while typing in a text input).
- [x] When no menu is open, `focus.menu_bar` focuses the first enabled trigger.
- [x] When a menu is open, `focus.menu_bar` closes it and focuses the active trigger.
- [x] Scriptable diagnostics can press `F10` to validate the flow.

Evidence anchors:

- Command + default keybindings: `crates/fret-app/src/core_commands.rs` (`FOCUS_MENU_BAR`).
- Declarative command hooks: `crates/fret-ui/src/{action.rs,elements/cx.rs,declarative/host_widget.rs}`.
- In-window menubar focus wiring: `ecosystem/fret-kit/src/{workspace_menu.rs,workspace_shell.rs}`.
- Demo wiring: `apps/fret-ui-gallery/src/driver.rs`.
- Diagnostics script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json`.

### MVP 3: macOS `NSMenu` Mapping (Done)

Goal: implement macOS `NSMenu` mapping with a minimal set of platform semantics, matching Zed’s
approach where feasible.

Deliverables:

- [x] Map `MenuBar` to a global `NSMenu` menubar (macOS is global; per-window is best-effort).
- [x] Map `MenuItem::Separator`, command items, and submenus.
- [x] System submenu support (macOS Services) via `MenuItem::SystemMenu` (ADR 0185).
- [x] Action dispatch must go through effects, not direct callbacks.
- [x] Shortcut display uses MVP 2 policy.
- [x] Enable/disable gating uses live `InputContext` snapshot where possible.
- [x] Shortcut dispatch can be gated via keymap `when` (e.g. `edit.can_undo/edit.can_redo`) so keyboard shortcuts match menu enablement.
- [x] Standard menu hooks use `MenuRole` (no title-string hacks for Window).
- [x] Minimal App menu baseline can be authored as commands (About/Preferences/Services/Hide/Hide Others/Show All/Quit).
  - Quit is implemented via `Effect::QuitApp` (see ADR 0001).
  - About can be implemented via `Effect::ShowAboutPanel` on macOS (ADR 0186).

Acceptance:

- [x] Switching active window does not require rebuilding the entire menubar every frame.
- [ ] Standard menus (App role expectations / default items) are handled (ADR 0185).
  - Current: the commands/effects exist and can be injected by workspace shells, but the final macOS App menu
    conventions (ordering, titles, and what becomes runner-native) are still open.

Reference anchors:

- Zed macOS menu creation: `repo-ref/zed/crates/gpui/src/platform/mac/platform.rs` (`create_menu_bar`, `create_menu_item`).

Evidence anchors:

- macOS mapping: `crates/fret-launch/src/runner/desktop/macos_menu.rs`.
- Desktop runner wiring: `crates/fret-launch/src/runner/desktop/mod.rs`.
- Menu open refresh hook: `crates/fret-launch/src/runner/desktop/app_handler.rs` (`validateMenuItem` / user event dispatch).
- Core app commands: `crates/fret-app/src/core_commands.rs` (`app.about`, `app.preferences`, `app.quit`).
- Quit effect: `crates/fret-runtime/src/effect.rs` (`Effect::QuitApp`).
- About effect: `crates/fret-runtime/src/effect.rs` (`Effect::ShowAboutPanel`).
- Golden-path default handling: `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (handles `app.quit`/`app.hide*`).
- ADR: `docs/adr/0186-macos-about-panel-effect.md`.
- Preferences policy: `docs/adr/0187-preferences-command-policy.md`.
- Workspace baseline supports App menu: `ecosystem/fret-workspace/src/menu.rs` (`WorkspaceMenuCommands`).
- Golden-path in-window default: `ecosystem/fret-kit/src/workspace_shell.rs` (`workspace_shell_model_default_menu`).
- Undo/Redo availability seam example: `apps/fret-examples/src/gizmo3d_demo.rs` (`sync_window_command_availability`).

### MVP 4: Linux Strategy (Decision) (Done)

Goal: decide how to support Linux OS menubars without committing to a heavyweight toolkit early.

Deliverables:

- [x] Document constraints and strategy: GTK/DBus menubar vs in-window fallback (ADR 0184).
- [ ] Prototype a minimal path for at least one distro stack (or explicitly defer with rationale).

Acceptance:

- [x] Clear decision recorded with trade-offs and user impact (ADR 0184).

Evidence anchors:

- ADR 0184: `docs/adr/0184-linux-menubar-strategy.md`.
- Current Linux behavior: `crates/fret-launch/src/runner/desktop/mod.rs` (non-Windows/macOS no-op for `Effect::SetMenuBar`).

## Recommended Next Steps (Post-MVP)

1) **Finalize macOS App menu conventions**
   - Lock ordering + wording for About/Preferences/Services/Hide/Hide Others/Show All/Quit.
   - Decide which items remain command-driven (preferred) vs which become runner-native behaviors.
   - Reference: Zed app menu mapping `repo-ref/zed/crates/gpui/src/platform/app_menu.rs`.

2) **Polish quit UX (optional)**
   - Current baseline: `QuitApp` is mediated by `before_close_window` and then force-closes all windows before exiting.
   - Follow-up candidates: decide whether "Quit" should always prompt in the main window (even if another window is key),
     and whether apps should be able to customize the prompt window selection policy.

3) **Decide Linux OS menubar investment level**
   - Either prototype a minimal integration for one stack (e.g. KDE/DBus global menu), or explicitly defer and document user impact.

### MVP 5: Advanced Semantics (Longer-term)

- [ ] Checked/radio state, and the state source-of-truth (command/meta vs menu item state).
- [ ] Dynamic submenus (e.g. Recent) with a stable update model (no plugin API required yet).
- [ ] Menu contribution API + user override merge rules (Unity-style customization).

## Risks / Design Notes

- The OS menubar is an integration surface with platform-specific semantics; avoid inflating the
  `fret-ui` runtime contract.
- Keep "shortcut display" and "enablement gating" as separate concerns, as Zed does implicitly:
  one is a UX hint, the other is a behavioral guardrail.
- Docking and multi-viewport are orthogonal: the key integration seam is window-scoped, not viewport-scoped.
