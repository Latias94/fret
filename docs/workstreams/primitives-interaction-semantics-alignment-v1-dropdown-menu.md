# Primitives Interaction Semantics Alignment v1 — DropdownMenu (Audit Sheet)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Active (workstream note; not a contract)

Baseline: Radix Dropdown Menu outcomes (including submenu pointer-grace semantics).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/dropdown-menu/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/dropdown_menu.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`

Key implementation anchors (submenu + grace corridor):

- Shared menu substrate:
  - `ecosystem/fret-ui-kit/src/primitives/menu/root.rs` (`with_root_name_sync_root_open_and_ensure_submenu`)
  - `ecosystem/fret-ui-kit/src/primitives/menu/sub.rs` (submenu open/switch policy + grace timer)
  - `ecosystem/fret-ui-kit/src/primitives/menu/pointer_grace_intent.rs` (corridor geometry + “moving towards submenu” tests)
- Shared safe-hover geometry:
  - `ecosystem/fret-ui-kit/src/headless/safe_hover.rs` (`safe_hover_contains`)

Key implementation anchors (dismiss + modality-gated focus):

- Overlay request + modal/click-through control:
  - `ecosystem/fret-ui-kit/src/primitives/menu/root.rs` (`dismissible_menu_request_with_modal*`)
  - tests: `menu_modal_controls_underlay_pointer_blocking_and_click_through`
- Close auto-focus suppression for non-modal outside-press dismissal (click-through):
  - `ecosystem/fret-ui-kit/src/primitives/menu/root.rs` (`menu_close_auto_focus_guard_hooks`)
- Initial focus gating by last input modality:
  - `ecosystem/fret-ui-kit/src/primitives/menu/root.rs` (menu initial focus targets; test `menu_request_gates_initial_focus_by_modality`)

Key implementation anchors (open keys):

- Trigger open-on-arrow behavior:
  - `ecosystem/fret-ui-kit/src/primitives/menu/trigger.rs` (`wire_open_on_arrow_keys`)

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `dropdown-menu-example.dropdown-menu.submenu-arrowleft-escape-close.light`
  - `dropdown-menu-example.dropdown-menu.outside-click-close.light`
  - `dropdown-menu-example.dropdown-menu.submenu-outside-click-close.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `dropdown-menu-example.dropdown-menu.open-navigate-select.light`

Evidence (Fret unit tests at recipe layer; fast invariants):

- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`:
  - `dropdown_menu_arrow_up_opens_and_focuses_last_item`
  - `dropdown_menu_item_select_closes_and_restores_trigger_focus`
  - `dropdown_menu_outside_press_closes_without_overriding_underlay_focus`
  - `dropdown_menu_non_modal_outside_press_closes_without_restoring_focus_to_trigger`
  - `dropdown_menu_submenu_does_not_switch_while_pointer_moves_through_safe_corridor`
  - `dropdown_menu_submenu_keyboard_open_transfers_focus_and_arrow_left_restores_focus`
  - `dropdown_menu_submenu_items_propagate_test_ids`

Scripted repros (existing):

- `tools/diag-scripts/ui-gallery-dropdown-open-select.json`
- `tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json`
- `tools/diag-scripts/ui-gallery-dropdown-menu-docs-smoke.json`
- `tools/diag-scripts/ui-gallery-dropdown-submenu-bounds.json`
- `tools/diag-scripts/ui-gallery-dropdown-submenu-underlay-dismiss.json`
- `tools/diag-scripts/ui-gallery-dropdown-submenu-arrowleft-escape-close.json` (keyboard submenu close)
- `tools/diag-scripts/ui-gallery-dropdown-submenu-safe-corridor-sweep.json` (corridor guard)

---

## Outcome model (what we must preserve)

State:

- `open` per menu root
- active/highlighted item (roving focus)
- “submenu open” state and pointer grace corridor state

Reasons:

- open: trigger press / keyboard open keys
- dismiss: escape / outside press / focus outside / scroll
- selection: item activate (press/enter) vs highlight-only

Invariants:

- Submenu does not immediately close when pointer moves diagonally through the “safe corridor”.
- Keyboard navigation is deterministic and does not depend on pointer state.
- Outside interaction handling is explicit (consume vs click-through), and consistent with Radix outcomes.

### State machine sketch (root + submenu)

This is a compact “reason → outcome” model aligned with Radix `@radix-ui/react-menu` /
`@radix-ui/react-dropdown-menu`.

Root session:

- Closed → Opened when:
  - Trigger is activated (press/click), or
  - Keyboard open key (ArrowDown/ArrowUp) is pressed on the trigger (`wire_open_on_arrow_keys`).
- Opened → Closed when:
  - Escape dismiss, or
  - Outside press / focus outside / scroll dismissal, or
  - Item selection commit (activation) chooses to close.

Submenu session (while root is open):

- SubmenuClosed(item) → SubmenuOpen(item) when:
  - Pointer enters a submenu trigger item and hover-open delay elapses (if configured), or
  - Keyboard navigation selects submenu trigger and ArrowRight opens.
- SubmenuOpen(item A) → SubmenuOpen(item B) when:
  - Pointer enters a different submenu trigger item (switch), unless suppressed by the pointer grace corridor.
- SubmenuOpen(item) → SubmenuClosed(item) when:
  - ArrowLeft closes, restoring focus to the submenu trigger, or
  - Root closes, or
  - Outside press dismiss closes the whole menu stack as configured.

Pointer grace corridor (submenu safe-hover):

- When the pointer leaves a submenu trigger towards the submenu content, we start a “grace window”
  during which submenu switching is suppressed if the pointer stays within the safe corridor.
- Geometry + membership checks:
  - corridor geometry: `ecosystem/fret-ui-kit/src/primitives/menu/pointer_grace_intent.rs`
  - “moving toward submenu” tests: `ecosystem/fret-ui-kit/src/headless/safe_hover.rs`

### Policy surfaces (where decisions live)

- Trigger open keys: `ecosystem/fret-ui-kit/src/primitives/menu/trigger.rs`
- Roving focus + typeahead policy: `ecosystem/fret-ui-kit/src/primitives/menu/content.rs`
- Close auto-focus suppression (click-through non-modal menus): `ecosystem/fret-ui-kit/src/primitives/menu/root.rs`

---

## Audit checklist (dimension-driven)

### Model

- [x] `M` Write down root + submenu state machine, including pointer grace corridor.

### Policy (Trigger / Listbox / Commit)

- [x] `M/I` TriggerPolicy: open/close inputs and open keys.
- [x] `M/I` ListboxPolicy: roving focus + typeahead + scroll-into-view.
- [ ] `M/I` SelectionCommitPolicy: activation commits selection and closes as appropriate.

### Focus

- [ ] `M/I` Focus behavior between trigger/content/submenu; restore on close.

### Dismiss

- [ ] `M/I` Escape/outside press/focus outside dismissal semantics (root vs submenu).

### Pointer

- [ ] `M/I` Pointer grace corridor semantics for submenus.
- [ ] `M/I` Click-through vs barrier behavior is deliberate and tested.

### Keys

- [ ] `M/I` Arrow navigation + typeahead match Radix outcomes; left/right manage submenu.

### A11y (semantics)

- [ ] `M/I` Menu/list semantics and disabled/checked/radio outcomes map correctly to AccessKit.

### Placement / size

- [ ] `M/I` Placement, collision, and submenu positioning outcomes match upstream.

### Time

- [ ] `M/I` Any hover-intent / grace delays are `Duration` and semantic.

### Tests / gates

- [x] `G` At least one multi-step diag script gates submenu corridor behavior.
- [x] `G` Keep a diag script gating keyboard submenu close (ArrowLeft) and root close (Escape).
- [ ] `G` Add/keep Radix timeline parity gates where available.
