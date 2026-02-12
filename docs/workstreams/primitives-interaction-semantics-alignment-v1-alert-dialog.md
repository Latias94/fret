# Primitives Interaction Semantics Alignment v1 — AlertDialog (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix AlertDialog outcomes (modal + “least destructive” focus defaults).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/alert-dialog/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`

Key implementation anchors (least-destructive focus preference):

- AlertDialog options surface:
  - `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs` (`AlertDialogRoot`, `AlertDialogOptions`)
- Barrier is non-dismissable by outside press (Radix-shaped):
  - `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs` (`alert_dialog_modal_barrier`, `alert_dialog_modal_request_with_options`)

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `alert-dialog-example.alert-dialog.open-cancel.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `alert-dialog-example.alert-dialog.open-cancel.light`

Scripted repros (existing):

- `tools/diag-scripts/ui-gallery-overlay-modals-visible.json` (alert dialog open snapshot)
- `tools/diag-scripts/ui-gallery-alert-dialog-least-destructive-initial-focus.json` (least-destructive focus gate)

---

## Outcome model (what we must preserve)

Invariants:

- Initial focus defaults to the “cancel/least destructive” action (unless configured).
- Escape/outside press dismissal semantics match upstream expectations.
- Modal barrier semantics match Dialog (with AlertDialog-specific focus defaults).

---

## Audit checklist (dimension-driven)

- [ ] `M` Document initial focus default rules and how they map to Fret semantics.
- [x] `G` Add a focused diag script that asserts least-destructive initial focus (if missing).
