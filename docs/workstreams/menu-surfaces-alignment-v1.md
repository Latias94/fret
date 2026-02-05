# Menu Surfaces Alignment v1 (OS menubar + in-window menubar + context menus)

Status: Draft (workstream note; not an ADR)
Last updated: 2026-02-05

This workstream is about preventing **behavior drift** between Fret’s menu surfaces:

- OS menubars (runner-owned mapping): Windows `HMENU`, macOS `NSMenu`
- In-window menubars (overlay + roving): workspace shells and app chrome
- Context menus and dropdown menus (overlay wrappers)
- Downstream surfaces that consume menus indirectly (command palette breadcrumbs, help, docs)

It complements (but does not replace) existing ADRs and workstreams:

- Data-only menu model: ADR 0023 (`docs/adr/0023-command-metadata-menus-and-palette.md`)
- OS menubar seam: ADR 0183 (`docs/adr/0183-os-menubar-effect-setmenubar.md`)
- OS menubar tracker: `docs/workstreams/os-menubar.md`
- Overlay policy split: ADR 0067 (`docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`)
- Focus + command routing: ADR 0020 (`docs/adr/0020-focus-and-command-routing.md`)

Tracking:

- TODO tracker: `docs/workstreams/menu-surfaces-alignment-v1-todo.md`

---

## 1) Motivation

Editor-grade applications treat menus as a **first-class keyboard surface**, not just a click UI.
If OS and in-window menus diverge (enable/disable gating, shortcut labels, focus behavior, submenu
intent/grace, separator trimming), users lose trust quickly:

- A command is disabled in the OS menubar but enabled in-window.
- Shortcut labels change unpredictably as focus moves.
- Menus click-through underlying UI, causing accidental edits.
- Keyboard opening focuses the wrong element and navigation feels “off”.

Fret already has the right architectural split:

- `crates/fret-runtime::menu` is **data-only**.
- `crates/fret-runtime::window_command_gating` provides a **cross-surface** gating snapshot.
- OS menubars are mapped in runners (`crates/fret-launch`).
- In-window menus are ecosystem-owned overlays (`ecosystem/fret-ui-kit`, `ecosystem/fret-kit`,
  `ecosystem/fret-ui-shadcn`).

This workstream locks down the “menu surfaces should agree” outcomes without pushing policy into
`crates/fret-ui`.

---

## 2) Reference models (what “good” looks like)

Local snapshots we can audit:

- Zed (GPUI)
  - OS menubars (`App::set_menus`) and client-side application menus for non-macOS.
  - Evidence: `repo-ref/zed/crates/gpui/src/app.rs` (`set_menus`)
  - Evidence: `repo-ref/zed/crates/title_bar/src/application_menu.rs` (hover switches open menu)
- Godot
  - Embedded `MenuBar` that can prefer a global/native menu bar when supported.
  - Evidence: `repo-ref/godot/scene/gui/menu_bar.cpp` (`set_prefer_global_menu`, hover/pressed state)
- Radix Menubar/Menu (behavioral outcomes)
  - We align outcomes via `ecosystem/fret-ui-kit::primitives::menu` / `menubar`.

Key takeaway from Zed: **shortcut display is a UX decision**, not a strictly “effective binding in
the current focus context”. That matches Fret’s `Keymap::display_shortcut_for_command_sequence`.

---

## 3) Current state (2026-02)

### 3.1 OS menubar

Tracked in `docs/workstreams/os-menubar.md`.

Baseline is implemented for Windows + macOS:

- Menu structure is data-only (`fret-runtime::MenuBar`).
- Shortcut labels come from keymap reverse lookup.
- Best-effort enable/disable uses a window-scoped gating snapshot.

### 3.2 In-window menubar

The canonical bridge from `fret-runtime::MenuBar` → in-window overlay menubar lives in:

- `ecosystem/fret-kit/src/workspace_menu.rs` (`menubar_from_runtime_with_focus_handle`)

Focus integration exists:

- Command `focus.menu_bar` (default F10 on Windows/Linux) is defined in
  `crates/fret-app/src/core_commands.rs`.
- Window-scoped gating for “menu bar is present” is modeled by
  `crates/fret-runtime/src/window_menu_bar_focus.rs` and used by workspace shells.
- Alt activation (Windows/Linux): Alt-up (press + release Alt without other keys) emits
  `focus.menu_bar` when `WindowMenuBarFocusService.present == true` and when `!focus.is_text_input`.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs` (`handle_alt_menu_bar_activation`)

Recently fixed (correctness + parity):

- In-window menubar now uses `WindowCommandGatingSnapshot` to determine enabled/disabled state for
  menu items, matching OS menubar behavior.
- In-window menubar sanitizes entries (separator trimming, empty submenu removal), aligning with
  Zed’s “sanitize menu items” behavior.

Evidence:

- `ecosystem/fret-kit/src/workspace_menu.rs` (gating + sanitize)

---

## 4) Invariants (do not break)

1) **No policy leakage into `crates/fret-ui`**
   - Menu interactions are policy-heavy; keep them in ecosystem crates.

2) **One data model**
   - `fret-runtime::MenuBar/MenuItem` remains the shared menu structure contract.
   - Customization continues to be layered via `menubar.json` (ADR 0014 / ADR 0183).

3) **One gating model**
   - All menu surfaces use `WindowCommandGatingSnapshot` (or best-effort snapshot helpers) for
     enablement decisions.

4) **Stable shortcut labels**
   - Use `Keymap::display_shortcut_for_command_sequence` (stable display contexts) for labels; do
     not make labels depend on live focus state.

---

## 5) Scope / non-goals

In scope:

- Align enable/disable gating across OS + in-window + context menus.
- Align separator trimming and empty-submenu handling across surfaces.
- Align keyboard-open initial focus outcomes (pointer vs keyboard modality).
- Define editor-grade keyboard navigation outcomes for in-window menubars (F10, Alt activation,
  arrow navigation, typeahead).
- Prepare for richer semantics (checked/radio) and dynamic menus (Recent, Window list).

Non-goals:

- Making menus a component library in `fret-ui`.
- Replacing command palette / docking / overlays with menu-driven UI.
- Pixel-perfect parity with any upstream; we gate outcomes and invariants.

---

## 6) Proposed milestones (v1)

Milestones and executable TODOs live in:

- `docs/workstreams/menu-surfaces-alignment-v1-todo.md`

High-level intent:

- **M0 — Correctness + baseline parity**
  - Gating parity (enabled/disabled) across OS and in-window.
  - Shared sanitize rules (separators, empty submenus).
  - Modal/click-through defaults for menu overlays.
  - Keyboard-open initial focus outcomes match Radix-like expectations.

- **M1 — Editor-grade keyboard UX**
  - Alt activation and mnemonic strategy (Windows/Linux).
  - F10 focus behavior and escape/back handling.
  - Robust roving + typeahead outcomes (menubar triggers and menu items).

- **M2 — Semantics and dynamic content**
  - Checked/radio state ownership decisions.
  - Dynamic submenus (Recent, Window list) with a stable update model.
  - Contribution/patch strategy for plugins without exploding contracts.

- **M3 — Regression gates**
  - `cargo nextest` unit tests for sanitize/gating/focus outcomes.
  - `fretboard diag` scripts for keyboard-only navigation and submenu grace intent.

---

## 7) Risks / design notes

- **Cross-surface divergence risk**: if sanitize/gating code exists in multiple places, drift is
  guaranteed. Prefer centralizing “menu normalization” into a shared helper (data-only if possible).
- **Alt/mnemonics**: introducing mnemonics touches input dispatch and localization; keep the first
  milestone minimal and outcome-driven.
- **Dynamic menus**: “Recent” and “Window list” are easy to implement badly (unstable IDs, rebuild
  churn). We should define ID stability and update frequency early.
