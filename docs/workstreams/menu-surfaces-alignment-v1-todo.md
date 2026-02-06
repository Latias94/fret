# Menu Surfaces Alignment v1 — TODO Tracker

Status: Draft
Last updated: 2026-02-06

This tracker covers the work described in:

- `docs/workstreams/menu-surfaces-alignment-v1.md`

Related:

- OS menubar tracker: `docs/workstreams/os-menubar.md`
- Menu model + command metadata: `docs/adr/0023-command-metadata-menus-and-palette.md`
- OS menubar seam: `docs/adr/0183-os-menubar-effect-setmenubar.md`
- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- Overlay policy: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`

Legend:

- [ ] open
- [~] in progress
- [x] done
- [!] blocked / needs decision

Tracking format:

- ID: `MENU-MVP{n}-{area}-{nnn}`
- Areas:
  - `parity` (OS vs in-window correctness parity)
  - `sanitize` (separator/empty submenu normalization)
  - `gating` (enable/disable, action availability, input context)
  - `kbd` (keyboard UX: F10/Alt/roving/typeahead)
  - `sem` (semantics: checked/radio, roles, a11y)
  - `dyn` (dynamic menus: Recent, Window list)
  - `conf` (menubar.json schema + merge policy)
  - `test` (nextest/diag gates)
  - `docs` (notes and decisions)

---

## M0 — Correctness + baseline parity (ship outcomes first)

Exit criteria:

- In-window menus and OS menus agree on enable/disable gating for the same `MenuBar`.
- Menu structure is normalized consistently (separators and empty submenus).
- Menu overlay does not click-through by default.
- Keyboard-open initial focus is sane and consistent (first item for keyboard, content container for pointer).

- [x] MENU-MVP0-gating-001 Use `WindowCommandGatingSnapshot` for in-window menu enable/disable.
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (`command_item`, `menubar_from_runtime_with_focus_handle`)
- [x] MENU-MVP0-sanitize-002 Sanitize in-window menu entries (drop duplicate/leading/trailing separators; drop empty submenus).
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (`sanitize_entries` + unit tests)
- [x] MENU-MVP0-parity-003 Make in-window menubar overlay modal by default (prevent click-through).
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (`dismissible_menu_request_with_modal_and_dismiss_handler(..., modal=true)`)
- [x] MENU-MVP0-kbd-004 Keyboard-open initial focus prefers first enabled item, otherwise content root.
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (first-item focus tracking)

- [x] MENU-MVP0-sanitize-010 Centralize menu normalization so OS and in-window share the same sanitize rules.
  - Evidence: `crates/fret-runtime/src/menu.rs` (`MenuBar::normalize`, `normalize_menu_items`)
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (normalizes before building in-window entries)
- [x] MENU-MVP0-parity-011 Ensure Windows/macOS OS menubar mapping also applies sanitize consistently (no trailing separators, no empty submenus).
  - Evidence: `crates/fret-launch/src/runner/desktop/windows_menu.rs` (normalizes before building `HMENU`)
  - Evidence: `crates/fret-launch/src/runner/desktop/macos_menu.rs` (normalizes before building `NSMenu`)
- [x] MENU-MVP0-gating-012 Add a unit test that asserts widget-scope action availability disables a menu item consistently across surfaces.
  - Evidence: `crates/fret-runtime/src/window_command_gating.rs` (`action_availability_disables_widget_scope_commands_only`)

---

## M1 — Editor-grade keyboard UX (F10/Alt/roving/typeahead)

Exit criteria:

- F10 focuses the in-window menubar when present, and no-ops cleanly otherwise.
- Alt activates the menubar on Windows/Linux (or a deliberate alternative is documented).
- Arrow and typeahead navigation matches the intended Radix/APG outcomes.

- [x] MENU-MVP1-kbd-020 Define and implement Alt activation strategy for in-window menubar (Windows/Linux).
  - Notes:
    - Decision: Alt-up (press + release Alt without other keys) triggers `focus.menu_bar` (does not auto-open the first menu).
    - Cancelation: any other keydown, pointer-down, or IME/text input while Alt is held cancels activation.
    - Guard: only fires when the window has an in-window menubar (`WindowMenuBarFocusService.present == true`) and when `!focus.is_text_input`.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs` (`handle_alt_menu_bar_activation`)
  - Evidence: `crates/fret-ui/src/tree/tests/alt_menu_bar_activation.rs` (nextest coverage)
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (publishes `WindowMenuBarFocusService` for in-window menubar)
- [x] MENU-MVP1-kbd-021 Implement mnemonic strategy (underlined letter + Alt+Key) or explicitly defer with rationale.
  - Decision (defer): we intentionally do **not** ship heuristics like “first letter of menu title”.
    Without a source-of-truth for mnemonics (localization, collisions, author overrides),
    heuristics create unstable UX and regressions across downstream apps.
  - Implemented:
    - explicit `Menu.mnemonic` + `menubar.json` v2 support (contract)
    - Alt+Key routing for in-window menubars (policy)
  - Evidence: `crates/fret-runtime/src/menu.rs` (`Menu.mnemonic`, `MenuFileV2.mnemonic`, patch ops v2)
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs` (`open_on_alt_mnemonic`)
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs` (`open_on_mnemonic_when_active`)
  - Evidence: `ecosystem/fret-kit/src/workspace_shell.rs` (installs Alt+mnemonic key handler)
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (installs Alt+mnemonic key handler; adds Gallery mnemonic)
  - Evidence: `crates/fret-launch/src/runner/desktop/windows_menu.rs` (maps mnemonics to Win32 `&` labels for OS menubar)
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-alt-mnemonic.json`
- [x] MENU-MVP1-kbd-026 Render mnemonic underlines for in-window menubar triggers (presentation).
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (`attributed_title_with_mnemonic_underline`, trigger label rendering)
- [x] MENU-MVP1-kbd-027 Escape exits "active menubar" state when no menu is open (restore focus).
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs` (`exit_active_on_escape_when_closed`)
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-escape-exits-active.json`
- [x] MENU-MVP1-kbd-022 Ensure Escape closes submenu, then menu, then returns focus to trigger (predictable unwind).
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (test `escape_unwinds_submenu_then_menu_and_restores_focus`)
- [x] MENU-MVP1-kbd-023 Add “menu key” (context-menu key / Shift+F10) alignment notes for consistency with context menus.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menu/trigger.rs` (`wire_open_on_shift_f10`)
  - Evidence: `ecosystem/fret-ui-shadcn/src/context_menu.rs` (wires Shift+F10)
  - Evidence: `tools/diag-scripts/ui-gallery-context-menu-shift-f10.json`
- [x] MENU-MVP1-parity-024 Switch open menus on hover (Zed/Godot-style `switch_on_hover`) with a small delay.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs` (`DEFAULT_HOVER_SWITCH_DELAY`, hover switch timer)
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-hover-switch.json`
- [x] MENU-MVP1-parity-025 Implement submenu “grace intent” (diagonal pointer travel tolerance); cover via `MENU-MVP4-test-052`.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menu/sub.rs` (pointer grace intent + close-delay timers)
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (installs submenu pointer-move handler for menubar overlays)
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-pointer-submenu-grace-intent.json`

---

## M2 — Semantics (checked/radio, a11y, roles)

Exit criteria:

- Checked/radio semantics have a source-of-truth and can be rendered consistently.
- A11y roles/labels are stable across OS and in-window, where possible.

- [x] MENU-MVP2-sem-030 Decide where checked/radio state lives.
  - Decision: state lives on the menu item model (`MenuItem::Command.toggle`), so OS and in-window
    surfaces share one source of truth.
  - Evidence: `crates/fret-runtime/src/menu.rs` (`MenuItemToggle`, `MenuItem::Command.toggle`)
- [x] MENU-MVP2-sem-031 Extend `MenuItem` model (or add a parallel “presentation” layer) for checked/radio/accelerator hints.
  - Implemented: `MenuItemToggleKind` + `MenuItemToggle` + `MenuItem::Command.toggle`
  - Evidence: `crates/fret-runtime/src/menu.rs`
- [x] MENU-MVP2-sem-032 Add a minimal in-window rendering for checked/radio indicators (no OS mapping required at first).
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (checkbox/radio semantics roles + checkmark indicator)

---

## M3 — Dynamic menus (Recent, Window list, contributions)

Exit criteria:

- Dynamic submenus can update without unstable IDs or excessive rebuild churn.
- Contribution/override strategy is defined without turning menus into a plugin API prematurely.

- [x] MENU-MVP3-dyn-040 Define a stable “dynamic submenu” contract (IDs, update frequency, ordering rules).
  - Implemented (MVP):
    - placeholder rows use data-only `MenuItem::Label` (e.g. `No recent items`), rendered consistently
      as disabled text across in-window and OS menubars;
    - dynamic actionable rows use command IDs with stable prefixes
      (`ui_gallery.recent.open.{n}`, `ui_gallery.window.activate.{n}`),
      so they remain addressable by command surfaces while keeping menu topology app-owned.
  - Contract (MVP):
    - Stable anchors: app baseline owns long-lived submenu anchors (`File > Recent`, `Window > Windows`).
    - Stable addressing: `menubar.json` patch targets submenus by path (`menu: ["File", "Recent"]`) and
      non-command rows by typed selectors (`{"type":"label","title":"..."}`), with index fallback.
    - Update frequency: menu trees rebuild on explicit state transitions (command handlers / window-registry changes),
      not on every frame.
    - Identity refresh: in-window menubar identity is sequence-keyed on menu updates to avoid stale subtree retention.
    - Ordering: dynamic lists are deterministic (`Recent` newest-first capped list; `Window` sorted + `Window N` labels).
    - Dynamic command metadata: command titles are synchronized on menu rebuild so command-backed dynamic
      rows display user-facing labels (`Recent N`, `Window N`) instead of raw IDs.
  - Evidence: `crates/fret-runtime/src/menu.rs` (`MenuItem::Label`)
  - Evidence: `crates/fret-runtime/src/menu.rs` (`MenuItemFileV2::Label`, `ItemSelectorTyped::Label`)
  - Evidence: `ecosystem/fret-kit/src/workspace_menu.rs` (in-window bridge for `Label`)
  - Evidence: `crates/fret-launch/src/runner/desktop/windows_menu.rs` + `crates/fret-launch/src/runner/desktop/macos_menu.rs` (OS mapping for `Label`)
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (`menu_bar_seq` keyed refresh + dynamic command prefixes/metadata sync)
- [x] MENU-MVP3-dyn-041 Implement `Recent` menu MVP (placeholder list + disabled state + later wiring).
  - Implemented: stable `File > Recent` submenu anchor with placeholder; when recent entries exist,
    UI Gallery emits command-backed dynamic rows (`ui_gallery.recent.open.{n}`) and handles activation
    through the normal command routing path.
  - Evidence: `ecosystem/fret-workspace/src/menu.rs` (default `Recent` submenu anchor)
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (`UiGalleryRecentItemsService` + dynamic menu rebuild)
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-recent-dynamic-updates.json`
- [x] MENU-MVP3-dyn-042 Implement a `Window` list MVP for multi-window apps (align with `MenuRole::Window` semantics).
  - Implemented: stable `Window > Windows` submenu anchor; UI Gallery derives a per-run stable `Window N` list from
    `UiGalleryHarnessDiagnosticsStore` (non-wasm), emits command-backed rows (`ui_gallery.window.activate.{n}`),
    and raises the target window via `WindowRequest::Raise`.
  - Evidence: `ecosystem/fret-workspace/src/menu.rs` (default `Windows` submenu anchor)
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (dynamic window list rebuild)
- [x] MENU-MVP3-conf-043 Decide how dynamic items interact with layered `menubar.json` patch ops (what is addressable?).
  - Decision (MVP): dynamic submenu customization is path+selector addressable, with stability guidance:
    - Submenu path: `menu: ["File", "Recent"]`
    - Dynamic placeholder rows use `type: "label"` in `menubar.json` v2
    - Non-command item targeting uses `remove_at` / `move_at_*` with typed selectors
      (`{"type":"submenu","title":"Recent"}`, `{"type":"label","title":"No recent items"}`),
      and falls back to index anchors when titles are not unique.
  - Recommendation:
    - Prefer stable submenu anchors for long-lived customization points (`Recent`, `Windows`).
    - Prefer index anchors for localized/duplicated labels; title selectors are convenience-only.
  - Evidence: `crates/fret-runtime/src/menu.rs` (`MenuTarget::Path`, `ItemSelectorTyped::{Submenu,Label}`, `MenuItemFileV2::Label`)
  - Evidence: `crates/fret-runtime/src/menu.rs` tests (`remove_at_can_remove_label_by_title`, `v2_replace_parses_label_items`)

---

## M4 — Regression gates (nextest + diag)

Exit criteria:

- Core invariants are covered by unit tests and at least one `fretboard diag` script.

- [x] MENU-MVP4-test-050 Add nextest coverage for menu normalization (shared helper, if implemented).
  - Evidence: `crates/fret-runtime/src/menu.rs` (tests `normalize_*`)
- [x] MENU-MVP4-test-051 Add a `fretboard diag` script for keyboard-only menubar navigation:
  - Focus menubar (F10)
  - Open menu (Enter/ArrowDown)
  - Rove items + open submenu + close with Escape
  - Verify stable `test_id` anchors
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-keyboard-navigation.json` (F10 focus → Window menu → Split submenu → Escape unwind)
  - Extra: `tools/diag-scripts/ui-gallery-menubar-alt-activation.json` (toggles in-window menubar on, then validates Alt activation focuses `menubar-trigger-file`)
- [x] MENU-MVP4-test-052 Add a diag script for pointer “submenu grace intent” (prevent accidental close when moving toward submenu).
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-pointer-submenu-grace-intent.json`
- [x] MENU-MVP4-test-053 Add a diag script for checked/radio semantics (role + checked flags) on in-window menubar items.
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-os-radio-checked.json`
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (updates menubar baseline when settings change)
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`role_is`, `checked_is` predicates)
- [x] MENU-MVP4-test-054 Add a diag script that proves dynamic menu content updates (Recent + Window list).
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-recent-dynamic-updates.json`
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`menu_select_path` intent step)
- [x] MENU-MVP4-test-055 Add a diag script that proves dynamic command-backed items execute and update state.
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-recent-window-commands.json`
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (`recent.open(...)`, `window.activate(...)`, `ui-gallery-status-last-action`)
- [x] MENU-MVP4-test-056 Add a diag assertion for `Window > Windows` checked/radio parity on active window.
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-recent-window-commands.json` (`checked_is` on `window.activate.1`)
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (`WindowFocusChanged` -> focused window tracking -> radio toggle mapping)
- [x] MENU-MVP4-test-057 Add a multi-window diag gate that proves `Window > Windows` radio checked state is mutually exclusive across `Window 1/2`.
  - Evidence: `tools/diag-scripts/ui-gallery-menubar-windows-radio-mutual-exclusive.json`
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (`ui_gallery.debug.window.open`, `window_create_spec`, dynamic `Window` submenu radio mapping)

---

## Notes / decisions log (keep short)

- [x] MENU-MVP0-docs-900 Pick the canonical layer for menu normalization (see `MENU-MVP0-sanitize-010`).
  - Decision: `crates/fret-runtime::menu` is the canonical normalization layer (`MenuBar::normalize`),
    and all surfaces must consume normalized menus before mapping/rendering.
  - Evidence: `crates/fret-runtime/src/menu.rs` (`MenuBar::normalize`, `MenuBar::normalized`)
- [x] MENU-MVP1-docs-901 Decide Alt activation behavior and document trade-offs.
  - Decision: Alt-up focuses menubar triggers via `focus.menu_bar` on Windows/Linux; see `MENU-MVP1-kbd-020`.
