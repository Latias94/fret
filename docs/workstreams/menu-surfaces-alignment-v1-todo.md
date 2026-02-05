# Menu Surfaces Alignment v1 — TODO Tracker

Status: Draft
Last updated: 2026-02-05

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
- [ ] MENU-MVP1-kbd-021 Implement mnemonic strategy (underlined letter + Alt+Key) or explicitly defer with rationale.
- [ ] MENU-MVP1-kbd-022 Ensure Escape closes submenu, then menu, then returns focus to trigger (predictable unwind).
- [ ] MENU-MVP1-kbd-023 Add “menu key” (context-menu key / Shift+F10) alignment notes for consistency with context menus.

---

## M2 — Semantics (checked/radio, a11y, roles)

Exit criteria:

- Checked/radio semantics have a source-of-truth and can be rendered consistently.
- A11y roles/labels are stable across OS and in-window, where possible.

- [ ] MENU-MVP2-sem-030 Decide where checked/radio state lives:
  - command meta (preferred for stable IDs), vs
  - menu item state (dynamic), vs
  - app-provided query hook (runner/UI reads from a service).
- [ ] MENU-MVP2-sem-031 Extend `MenuItem` model (or add a parallel “presentation” layer) for checked/radio/accelerator hints.
- [ ] MENU-MVP2-sem-032 Add a minimal in-window rendering for checked/radio indicators (no OS mapping required at first).

---

## M3 — Dynamic menus (Recent, Window list, contributions)

Exit criteria:

- Dynamic submenus can update without unstable IDs or excessive rebuild churn.
- Contribution/override strategy is defined without turning menus into a plugin API prematurely.

- [ ] MENU-MVP3-dyn-040 Define a stable “dynamic submenu” contract (IDs, update frequency, ordering rules).
- [ ] MENU-MVP3-dyn-041 Implement `Recent` menu MVP (placeholder list + disabled state + later wiring).
- [ ] MENU-MVP3-dyn-042 Implement a `Window` list MVP for multi-window apps (align with `MenuRole::Window` semantics).
- [ ] MENU-MVP3-conf-043 Decide how dynamic items interact with layered `menubar.json` patch ops (what is addressable?).

---

## M4 — Regression gates (nextest + diag)

Exit criteria:

- Core invariants are covered by unit tests and at least one `fretboard diag` script.

- [ ] MENU-MVP4-test-050 Add nextest coverage for menu normalization (shared helper, if implemented).
- [~] MENU-MVP4-test-051 Add a `fretboard diag` script for keyboard-only menubar navigation:
  - Focus menubar (F10)
  - Open menu (Enter/ArrowDown)
  - Rove items + open submenu + close with Escape
  - Verify stable `test_id` anchors
  - Partial: `tools/diag-scripts/ui-gallery-menubar-alt-activation.json` (toggles in-window menubar on, then validates Alt activation focuses `menubar-trigger-file`)
- [ ] MENU-MVP4-test-052 Add a diag script for pointer “submenu grace intent” (prevent accidental close when moving toward submenu).

---

## Notes / decisions log (keep short)

- [ ] MENU-MVP0-docs-900 Pick the canonical layer for menu normalization (see `MENU-MVP0-sanitize-010`).
- [x] MENU-MVP1-docs-901 Decide Alt activation behavior and document trade-offs.
  - Decision: Alt-up focuses menubar triggers via `focus.menu_bar` on Windows/Linux; see `MENU-MVP1-kbd-020`.
