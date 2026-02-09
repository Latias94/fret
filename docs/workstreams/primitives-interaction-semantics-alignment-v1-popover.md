# Primitives Interaction Semantics Alignment v1 — Popover (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix Popover outcomes (dismiss semantics and focus behavior; click-through defaults are a Fret policy choice).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/popover/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/popover.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/popover.rs`

Key implementation anchors (dismiss reasons → close auto-focus outcomes):

- `ecosystem/fret-ui-kit/src/primitives/popover.rs` (`popover_close_auto_focus_guard_hooks`)

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (overlay geometry gates)

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `popover-example.popover.open-close.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `popover-example.popover.open-close.light`

Scripted repros (existing; not exhaustive):

- `tools/diag-scripts/ui-gallery-overlay-portal-geometry-clamp.json`
- `tools/diag-scripts/ui-gallery-popover-dialog-escape-underlay.json`
- `tools/diag-scripts/ui-gallery-overlay-modals-visible.json` (popover open snapshot)

---

## Outcome model (what we must preserve)

State:

- `open`
- focus restore policy keyed by close reason

Invariants:

- Dismiss reason mapping is stable (escape/outside/focus outside).
- Placement/collision/clamping is stable under tight viewports.
- Interaction during close transition is explicit (no accidental modal barrier).

---

## Audit checklist (dimension-driven)

- [ ] `M` Document dismiss + focus restore outcomes, including `preventDefault`-style hooks.
- [ ] `G` Keep at least one diag script gating: clamp under scroll + escape/outside dismiss.
