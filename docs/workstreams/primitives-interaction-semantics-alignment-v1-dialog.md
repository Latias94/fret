# Primitives Interaction Semantics Alignment v1 — Dialog (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix Dialog outcomes (modal barrier, focus trap/restore, escape/outside dismissal).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/dialog/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/dialog.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/dialog.rs`

Key implementation anchors (modal barrier + dismiss routing):

- Modal barrier element + layout:
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (`modal_barrier_layout`, `modal_barrier*`)
- Barrier routing of outside-press dismissals through a handler:
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (test `modal_barrier_can_route_dismissals_through_handler`)
- Reason-aware close auto-focus guard + default-close dismissal wrapper:
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (`DialogCloseAutoFocusGuardPolicy`, `dialog_close_auto_focus_guard_hooks`)
  - `ecosystem/fret-ui-shadcn/src/dialog.rs` (uses the guard hooks when wiring `modal_dialog_request_with_options_and_dismiss_handler`)

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (dialog overlay gates)

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `dialog-example.dialog.open-close.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `dialog-example.dialog.open-close.light`

Scripted repros (existing; not exhaustive):

- `tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json`
- `tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json`
- `tools/diag-scripts/ui-gallery-dialog-docs-order-smoke.json`
- `tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json`
- `tools/diag-scripts/ui-gallery-overlay-torture.json`
- `tools/diag-scripts/ui-gallery-overlay-modals-visible.json` (dialog open snapshot)

---

## Outcome model (what we must preserve)

State:

- `open`
- focus trap state + restore policy
- modal barrier behavior (hit-testing, pointer occlusion)

Invariants:

- Escape closes when allowed; focus restores predictably and is reason-aware.
- Underlay interactions are blocked when modal (unless explicitly allowed during close transition).
- Focus trap respects “prevent close auto focus” outcomes.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document close reasons and focus restore policy (reason → outcome).
- [ ] `G` Keep a diag script gating: escape close + focus restore + underlay block.
