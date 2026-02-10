# Primitives Interaction Semantics Alignment v1 — ContextMenu (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix Context Menu outcomes (right-click / keyboard menu key / Shift+F10).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/context-menu.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/context-menu/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/context_menu.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/context_menu.rs`

Key implementation anchors (open reasons + anchoring):

- Keyboard open (Shift+F10 / ContextMenu key):
  - `ecosystem/fret-ui-kit/src/primitives/context_menu.rs` (`wire_context_menu_open_on_shift_f10`)
  - `ecosystem/fret-ui-kit/src/primitives/menu/trigger.rs` (`wire_open_on_shift_f10`)
- Pointer open (right click / macOS ctrl-click):
  - `ecosystem/fret-ui-kit/src/primitives/context_menu.rs` (`context_menu_pointer_down_policy`)
- Touch long-press open:
  - `ecosystem/fret-ui-kit/src/primitives/context_menu.rs` (`ContextMenuTouchLongPress` + `*_pointer_handlers`)
- Long-press delay is semantic (`Duration`):
  - `ecosystem/fret-ui-kit/src/primitives/context_menu.rs` (`CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY`)
  - tests: `touch_long_press_arms_timer_and_returns_anchor_on_fire`, `touch_long_press_clears_when_pointer_moves_far`, `touch_long_press_clears_on_pointer_cancel`
- Cursor anchoring store for open model:
  - `ecosystem/fret-ui-kit/src/primitives/context_menu.rs` (`context_menu_anchor_store_model`, `set_context_menu_anchor_for_open_model`)

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `context-menu-example.context-menu.submenu-arrowleft-escape-close.light`
  - `context-menu-example.context-menu.outside-click-close.light`
  - `context-menu-example.context-menu.submenu-outside-click-close.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `context-menu-example.context-menu.context-open-close.light`

Scripted repros (existing):

- `tools/diag-scripts/ui-gallery-context-menu-right-click.json`
- `tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json`
- `tools/diag-scripts/ui-gallery-context-menu-shift-f10.json`
- `tools/diag-scripts/ui-gallery-context-menu-overlay-right-click-open-close.json` (gate: open + select + closes)
- `tools/diag-scripts/ui-gallery-contextmenu-edge-bounds.json`

---

## Outcome model (what we must preserve)

State:

- `open`
- anchor point (pointer location vs trigger bounds) and derived placement
- roving focus highlight state

Reasons:

- open: pointer secondary click, keyboard (Shift+F10 / Menu key), programmatic
- dismiss: escape / outside press / focus outside / scroll

Invariants:

- Keyboard-open context menu anchors in a predictable place (usually trigger bounds), not at last pointer position.
- Right-click does not steal selection in the underlying surface unless policy says so.
- Edge placement clamps correctly in tight viewports.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document open reason → anchor semantics (pointer vs keyboard).
- [ ] `M/I` Dismiss semantics per reason (escape/outside/focus).
- [ ] `M/I` Pointer policies: prevent accidental drag selection changes on right click.
- [ ] `M/I` Placement clamping near edges (tight viewport).
- [x] `G` Keep at least one diag script gating right-click + Shift+F10 outcomes.
