# Primitives Interaction Semantics Alignment v1 — Tooltip (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix Tooltip outcomes (provider delays, skip-delay window, pointer/keyboard triggers).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/tooltip/src/*`

---

## Current Fret implementation anchors

- Primitives/policy:
  - `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`
  - `ecosystem/fret-ui-kit/src/primitives/tooltip_provider.rs`
  - delay-group state machine: `ecosystem/fret-ui-headless/src/tooltip_delay_group.rs`
  - shared hover intent helper: `ecosystem/fret-ui-kit/src/primitives/hover_intent.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/tooltip.rs`

Time-source note (audit focus):

- The current Tooltip surface is **tick-based** (`open_delay_ticks*` / `close_delay_ticks*`), with a deterministic delay-group state machine.
- The shadcn recipe configures delays in **ticks/frames**; misleading `*_ms` helpers were removed to avoid API names that do not match the underlying time source.
- The workstream principle is “delays are semantic”; expect follow-up work to lift these to
  `Duration` at the policy surface (or to clearly document “ticks” as the stable unit).

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` (tooltip fixtures)

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `tooltip-example.tooltip.hover-show-hide.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `tooltip-example.tooltip.hover-show-hide.light`

Scripted repros (existing):

- `tools/diag-scripts/ui-gallery-tooltip-repeat-hover.json`
- `tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json`
- `tools/diag-scripts/ui-gallery-overlay-modals-visible.json` (tooltip open snapshot)

---

## Outcome model (what we must preserve)

State:

- `open`
- provider delay state (open delay, skip-delay window)
- pointer hover intent state (enter/leave, grace windows if any)

Invariants:

- Provider delays are semantic (`Duration`) and consistent across instances.
- Keyboard focus-triggered tooltip is distinct from hover-triggered tooltip.
- Scroll/viewport clamp behavior is stable.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document provider + delay group semantics (delay, skip-delay window).
- [ ] `M/I` Ensure all delays are `Duration` and surfaced as policy configuration (not recipe magic).
- [x] `G` Keep a diag script gating repeat-hover stability and skip-delay behavior.
