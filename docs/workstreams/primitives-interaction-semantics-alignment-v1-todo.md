# Primitives Interaction Semantics Alignment v1 — TODO

Status: Active (workstream note; not a contract)

See also:

- Workstream plan: `docs/workstreams/primitives-interaction-semantics-alignment-v1.md`
- Milestones: `docs/workstreams/primitives-interaction-semantics-alignment-v1-milestones.md`

---

## Select (Radix baseline)

- [ ] Fill the audit sheet and link evidence anchors:
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-select.md`
- [ ] Document current Select behavior and knobs (what is already implemented) with file anchors:
  - `ecosystem/fret-ui-kit/src/primitives/select.rs`
  - `ecosystem/fret-ui-shadcn/src/select.rs`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- [ ] Introduce an explicit config type (policy surface) in `fret-ui-kit::primitives::select`:
  - `pointer_up_guard` (default on)
  - `mouse_up_selection_gate` (default off; only affects mouseup commit paths)
  - `cancel_open_on_mouseup_outside` (default off; close reason = `CancelOpen`)
- [ ] Add unit tests for:
  - pointer-up suppression when opened by mouse pointerdown
  - mouseup gate prevents commit without prior pointerdown on the item
  - cancel-open closes only on the matching mouseup outside bounds (with slop/offset)

---

## Combobox (Base UI baseline)

- [ ] Fill the audit sheet and link evidence anchors:
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-combobox.md`
- [ ] Audit upstream combobox composition:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/combobox.tsx`
  - identify which behaviors come from Base UI vs shadcn composition
- [x] Add `ecosystem/fret-ui-kit/src/primitives/combobox.rs`:
  - open/close reasons + mapping from Fret `DismissReason`
  - callback/event gating helpers (`open_change_events`, `value_change_event`)
  - (later) listbox policy helpers (active/highlight + typeahead/scroll-into-view)
- [ ] Refactor `ecosystem/fret-ui-shadcn/src/combobox.rs`:
  - move the state machine helpers into `fret-ui-kit::primitives::combobox`
  - leave recipe as styling + composition
- [x] Refactor `ecosystem/fret-ui-shadcn/src/combobox.rs`:
  - move the state machine helpers into `fret-ui-kit::primitives::combobox`
  - leave recipe as styling + composition
- [x] Add unit tests at primitives layer:
  - `onValueChange` fires only on actual value change
  - reason mapping: dismiss → `OpenChangeReason`
  - open-change “complete” events follow presence (no early complete while animating)

---

## Cross-cutting (policy + time)

- [ ] Ensure any delays are configured as `Duration` at the primitives layer.
- [ ] Add stable `test_id` anchors needed for scripted timeline repros.
- [ ] Add at least one `tools/diag-scripts/*.json` script per primitive and gate it via `fretboard diag run`.

---

## Overlay families (Radix baseline)

- [ ] Fill audit sheets (first pass) and add evidence anchors:
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-dropdown-menu.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-context-menu.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-menubar.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-navigation-menu.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-tooltip.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-hover-card.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-popover.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-dialog.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-alert-dialog.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-sheet.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-drawer.md`

- [x] Add missing diag scripts where needed (notably `NavigationMenu`): `tools/diag-scripts/ui-gallery-navigation-menu-hover-switch-and-escape.json`.

---

## Toast (Sonner baseline)

- [ ] Fill audit sheet and link evidence anchors:
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-toast.md`
