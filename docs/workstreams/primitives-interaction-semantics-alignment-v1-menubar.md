# Primitives Interaction Semantics Alignment v1 — Menubar (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix Menubar outcomes (keyboard-first activation, mnemonic/Alt behavior, hover switching).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/menubar.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/menubar/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/menubar.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/menubar.rs`

Shared menu substrate (dismiss + click-through semantics):

- `ecosystem/fret-ui-kit/src/primitives/menu/root.rs`:
  - `dismissible_menu_request_with_modal*` (modal vs click-through)
  - `menu_close_auto_focus_guard_hooks` (suppress close auto-focus for non-modal outside presses)

Key implementation anchors (trigger row active-mode + mnemonics):

- Trigger-row policy surface + unit tests:
  - `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
    - `open_on_alt_mnemonic`
    - `open_on_mnemonic_when_active`
    - `exit_active_on_escape_when_closed`
    - hover switch timer (`DEFAULT_HOVER_SWITCH_DELAY`, semantic `Duration`)

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`

Evidence (Radix web overlay geometry gate):

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `menubar-example.menubar.open-navigate-close.light`

Evidence (Fret unit tests at primitives layer; fast invariants):

- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs` (selection; non-exhaustive):
  - `alt_mnemonic_opens_menu_when_trigger_is_active_but_closed`
  - `mnemonic_without_alt_opens_menu_when_menubar_active_and_closed`
  - `mnemonic_without_alt_does_not_steal_typing_when_trigger_row_not_focused`
  - `escape_exits_active_menubar_when_closed_and_restores_focus`
  - `toggle_opens_when_active_but_closed`

Scripted repros (existing; not exhaustive):

- `tools/diag-scripts/ui-gallery-menubar-alt-activation.json`
- `tools/diag-scripts/ui-gallery-menubar-alt-mnemonic.json`
- `tools/diag-scripts/ui-gallery-menubar-active-mnemonic.json`
- `tools/diag-scripts/ui-gallery-menubar-escape-exits-active.json`
- `tools/diag-scripts/ui-gallery-menubar-hover-switch.json`
- `tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json`
- `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- `tools/diag-scripts/ui-gallery-menubar-keyboard-navigation.json`

---

## Outcome model (what we must preserve)

State:

- “menubar active” vs “inactive”
- active top-level menu, open submenu chain
- keyboard mnemonic/Alt activation state

Reasons:

- activate: Alt key, explicit click, programmatic
- dismiss: escape, outside press, focus outside

Invariants:

- Alt activation does not open a menu by accident; it enters “menubar active” mode.
- Hover switching only applies while active and consistent with platform expectations.
- Escape exits active mode and restores focus predictably.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document menubar active-mode state machine and reasons.
- [ ] `M/I` Key routing: Alt/mnemonic and arrow navigation are deterministic.
- [ ] `M/I` Pointer hover switching semantics are explicit and gated.
- [x] `G` Maintain at least one diag script covering: Alt activate → open → navigate → escape restore.
