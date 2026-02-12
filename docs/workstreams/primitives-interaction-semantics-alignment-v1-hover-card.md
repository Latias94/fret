# Primitives Interaction Semantics Alignment v1 — HoverCard (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix HoverCard outcomes (hover intent, open/close delays, focus interactions).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/hover-card/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/hover_card.rs`
- Shared hover-intent primitive (delay open/close): `ecosystem/fret-ui-kit/src/primitives/hover_intent.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/hover_card.rs`

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` (hover card fixtures)

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `hover-card-example.hover-card.hover.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `hover-card-example.hover-card.hover.light`

Scripted repros (existing):

- `tools/diag-scripts/ui-gallery-overlay-modals-visible.json` (hover card open snapshot)
- `tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json`
- `tools/diag-scripts/ui-gallery-hovercard-hover-delayed-close.json`
- `tools/diag-scripts/ui-gallery-ai-chat-demo-inline-citation-hovercard.json`

---

## Outcome model (what we must preserve)

State:

- `open`
- hover intent + delay state

Invariants:

- Hover intent/delays are semantic and deterministic.
- Focus interactions (tabbing into content) are defined and tested.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document hover intent + delay state machine and reasons.
- [ ] `M/I` Ensure delays are `Duration` and policy-level.
- [x] `G` Add/keep at least one diag script for: hover open → move away → delayed close.
